use crate::demo::{DemoFrame, Vector3};
use std::{f32::consts::PI};
use crate::bspfile::{BspData};

#[derive(Debug)]
pub struct JumpSegment<'a> {
    pub start_index: usize,
    pub end_index: usize,
    pub frames:  &'a [DemoFrame]
}

#[derive(Debug, Clone, Copy)]
pub struct MovementData {
    pub movement_type: u8,
    pub frame: i32
}



const CONTENTS_EMPTY: i32 = 0;
const CONTENTS_SOLID: i32 = -1;

pub fn is_point_walkable_clip(point: Vector3, bsp: &BspData) -> bool {
    let mut node_index = bsp.models[0].headnode[1]; // model[0], clipnode용

    loop {
        if node_index < 0 {
            let contents = -1 - node_index;
            return contents == CONTENTS_EMPTY;
        }

        let node = &bsp.clipnodes[node_index as usize];
        let plane = &bsp.planes[node.planenum as usize];

        let d = point.x * plane.normal[0] +
                point.y * plane.normal[1] +
                point.z * plane.normal[2] - plane.dist;

        let side = if d >= 0.0 { 0 } else { 1 };
        node_index = node.children[side] as i32;
    }
}


pub fn angle_vectors (angle:&Vector3) -> (Vector3, Vector3, Vector3) {
        let (pitch, yaw, roll) = (angle.x, angle.y, angle.z);

        let trans_angle = |deg: f32| deg * (PI * 2.0 / 360.0);

        let sy = trans_angle(yaw).sin();
        let cy = trans_angle(yaw).cos();

        let sp = trans_angle(pitch).sin();
        let cp = trans_angle(pitch).cos();

        let sr = trans_angle(roll).sin();
        let cr = trans_angle(roll).cos();

        let forward = Vector3 {
            x: cp * cy,
            y: cp * sy,
            z: -sp,
        };

        let right = Vector3 {
            x: -sr * sp * cy - cr * -sy,
            y: -sr * sp * sy - cr * cy,
            z: -sr * cp,
        };

        let up= Vector3 {
            x: cr * sp * cy - sr * -sy,
            y: cr * sp * sy - sr * cy,
            z: cr * cp,
        };
        
        (forward, right, up)
    }

pub fn friction(state: &DemoFrame) {

    let velocity = state.simvel;
    let speed: f32 = (velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z).sqrt();
    if speed < 0.1 {
        return;
    }

    let drop = 0;

    if state.onground {
       let start:Vector3;
       let stop:Vector3;
    }
    
}

pub fn accelerate(state: &DemoFrame) -> Vector3 {
    if !state.onground {
        return Vector3 {x:0.0,y:0.0,z:0.0};
    }

    let (forward, right, up) = angle_vectors(&state.viewangle);
    
    let wishvel = Vector3 {
        x: forward.x * state.forwardmove + right.x * state.sidemove + up.x * state.upmove,
        y: forward.y * state.forwardmove + right.y * state.sidemove + up.y * state.upmove,
        z: forward.z * state.forwardmove  + right.z * state.sidemove  + up.z * state.upmove
    };

    
     
    
    
    return Vector3  {x:0.0,y:0.0,z:0.0};
}

pub fn airaccelerate(state: &DemoFrame) -> Vector3 {
    if state.onground {
        return Vector3 {x:0.0,y:0.0,z:0.0};
    }

    let (forward, right, _up) = angle_vectors(&state.viewangle);
    
    let wishvel = Vector3 {
        x: forward.x * state.forwardmove + right.x * state.sidemove,
        y: forward.y * state.forwardmove + right.y * state.sidemove,
        z: forward.z * state.forwardmove + right.z * state.sidemove
    };

    let mut wishspd:f32 = wishvel.length();
    if wishspd > 30.0 {
        wishspd = 30.0;
    }

    let wishdir = wishvel.normalize();

    let currentspeed = state.simvel.dot(wishdir);

    let mut addspeed = wishspd - currentspeed;
    
    if addspeed <= 0.0 {
        addspeed = 0.0;
    }

    let mut accelspeed: f32 = state.airaccelerate * state.frametime * wishvel.length() * 1.0;
    
    if accelspeed > addspeed {
        accelspeed = addspeed;
    };
    
    // println!("{}",accelspeed);

    let final_speed = Vector3{
        x: state.simvel.x + accelspeed * wishdir.x,
        y: state.simvel.y + accelspeed * wishdir.y,
        z: state.simvel.y + accelspeed * wishdir.z
    };
    return final_speed
}

pub fn strafe_optimize(state: &DemoFrame) -> (f32, Vector3) {
    let mut max_speed = 0.0;
    let mut optimized_param = 0.0;

    let mut upper_view_y = (state.viewangle.y * 10.0 + 30.0) as i32;
    let mut lower_view_y = (state.viewangle.y * 10.0 - 30.0) as i32;
    if upper_view_y > 3600 {
        upper_view_y -= 3600;
    }
    if lower_view_y > 3600 {
        lower_view_y -= 3600;
    }

    if upper_view_y < 0 {
        upper_view_y += 3600;
    }
    if upper_view_y <0 {
        upper_view_y += 3600;
    }

    for viewangle_y in (lower_view_y..upper_view_y).step_by(1) {
        let viewangle_f:f32 = viewangle_y as f32 / 10.0;
        
        let mut test_state = state.clone();
        test_state.viewangle.y = viewangle_f;

        let final_speed = airaccelerate(&test_state);

        let len = final_speed.length();
        if len > max_speed {
            max_speed = len;
            optimized_param = viewangle_f;
        }
    }

    let original = airaccelerate(&state);

    return (max_speed, Vector3{x:state.viewangle.x, y:optimized_param, z:state.viewangle.z})
}


pub fn extract_jump_segments<'a>(frames: &'a[DemoFrame]) -> Vec<JumpSegment<'a>> {
    let segments = Vec::new();
    let mut i = 0;
    let first_jump = true;

    //movement 데이터를 저장할 벡터 [0: duck] [1:jump]
    let mut movements: Vec<MovementData> = Vec::new();
    
    //연속동작 여부
    let mut is_sequenced: bool = false;

    //fog 카운트
    let mut frames_on_ground: u8 = 0;

    while i < frames.len() {

        let (optimized_speed, optimized_viewangle) = strafe_optimize(&frames[i]);

        println!("optimized_speed: {}, optimized_viewangle: {:?}", optimized_speed, optimized_viewangle);
        //onground 동작부
        if frames[i].onground {
            //연속동작 중 onground일 때 경우 fog 증가
            if is_sequenced {
                frames_on_ground += 1;
            }

            //fog 10 초과면 초기화
            if frames_on_ground > 10 {
                is_sequenced = false;
                frames_on_ground = 0;
                movements = Vec::new()
            }
            i += 1;
            continue;
        }


        //duck 감지
        if frames[i-2].onground & frames[i-2].command.contains(&"+duck".to_string()) {
            if !is_sequenced {
                is_sequenced = true;
            }
            movements.push( MovementData {
                movement_type: 0,
                frame: frames[i-2].frame
            });
        }
        
        //jump 감지
        if frames[i-1].onground & frames[i-1].command.contains(&"+jump".to_string()) {
            if !is_sequenced {
                is_sequenced = true;
            }
            movements.push( MovementData {
                movement_type: 1,
                frame: frames[i-1].frame
            });
        }
        
        i += 1;

    };

    segments
}
