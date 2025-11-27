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

pub fn is_clipped(point: Vector3, bsp: &BspData) -> bool {
    let mut node_index = bsp.models[0].headnode[1]; // clipnode root

    loop {
        if node_index < 0 {
            let contents = node_index; // 그대로 사용
            return contents != CONTENTS_EMPTY;
        }

        if let Some(node) = bsp.clipnodes.get(node_index as usize) {
            if node.planenum < 0 || node.planenum as usize >= bsp.planes.len() {
                return true;
            }

            let plane = &bsp.planes[node.planenum as usize];
            let d = point.x * plane.normal[0]
                  + point.y * plane.normal[1]
                  + point.z * plane.normal[2]
                  - plane.dist;

            let side = if d >= 0.0 { 0 } else { 1 };
            node_index = node.children[side] as i32;
        } else {
            return true;
        }
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


pub fn pm_friction(state: &DemoFrame, map_data: &BspData) -> Vector3 {
    if !state.onground {
        return Vector3 { x: (0.0), y: (0.0), z: (0.0) }
    }
    let speed = state.simvel.length();

    let mut start: Vector3 = Vector3 {x:(0.0), y:(0.0), z:(0.0)};
    let mut stop: Vector3 = Vector3 {x:(0.0), y:(0.0), z:(0.0)};
    // println!("vh: {:?}, cmd: {:?},spd: {:?}", state.viewheight, state.command, speed);

    let mut custom_simorg = state.simorg;
    custom_simorg.z -= 400.0;
    let trace: bool = is_clipped(custom_simorg, map_data);
    //println!("trace: {}",trace);

    start.x = state.simorg.x + state.simvel.x * 16.0;
    start.y = state.simorg.y + state.simvel.y * 16.0;
    start.z = state.simorg.z + state.simvel.z * 16.0;

    if state.viewheight.z == 12.0 {
        //앉은 상태
    } else if state.viewheight.z == 17.0 {
        //서 있는 상태
    } else {
        //앉는 도중
    };

    Vector3 {x:(0.0), y:(0.0), z:(0.0)}
}

/// PM_PlayerTrace 의 단순화 버전.
/// 플레이어 진행 방향으로 16 유닛, 아래로 70+36 유닛 떨어진 지점을 샘플링해서
/// 그 지점이 공중(빈 공간)인지 여부와, 해당 지점의 월드 좌표를 반환한다.
///
/// edgefriction 은 이 점이 공중일 때 적용된다고 본다.
pub fn pm_player_trace(state: &DemoFrame, map_data: &BspData) -> (bool, Vector3) {
    // 시야각으로부터 진행 방향 벡터 계산
    let (forward, _right, _up) = angle_vectors(&state.viewangle);

    // XY 평면에서의 진행 방향만 사용 (위를/아래를 보는 각도는 무시)
    let mut dir_2d = Vector3 {
        x: forward.x,
        y: forward.y,
        z: 0.0,
    };

    // 정규화 (0 벡터인 경우를 방지)
    let len = dir_2d.length();
    if len != 0.0 {
        dir_2d.x /= len;
        dir_2d.y /= len;
    }

    // 플레이어 origin 기준으로 16 유닛 전방, 70+36 유닛 하방
    let mut sample_point = state.simorg;
    sample_point.x += dir_2d.x * 16.0;
    sample_point.y += dir_2d.y * 16.0;
    sample_point.z -= 70.0 + 36.0;

    // 해당 지점이 공중인지(빈 공간인지) 판정
    let clipped = is_clipped(sample_point, map_data);
    let is_edge = !clipped; // CONTENTS_EMPTY 일 때 edge 로 간주

    (is_edge, sample_point)
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


pub fn extract_jump_segments<'a>(frames: &'a [DemoFrame], map_data: &BspData) -> Vec<JumpSegment<'a>> {
    let mut segments: Vec<JumpSegment<'a>> = Vec::new();
    let mut i: usize = 0;

    // movement 데이터를 저장할 벡터 [0: duck] [1:jump]
    let mut movements: Vec<MovementData> = Vec::new();

    // 연속 동작 여부 (duck / jump 가 하나의 기술로 이어지는 중인지)
    let mut is_sequenced: bool = false;

    // fog(frames on ground) 카운트
    // 같은 기술로 간주되는 구간에서, 지면에 붙어 있는 프레임 수
    let mut frames_on_ground: u8 = 0;

    // 현재 기술의 시작/마지막 프레임 인덱스
    // start 는 "실제 기술 입력(duck/jump) 이전에 지상에서 가속하던 구간"까지 포함하도록
    // 뒤로 확장해서 잡는다.
    let mut current_segment_start: Option<usize> = None;
    let mut last_segment_index: usize = 0;

    while i < frames.len() {
        // 현재 프레임 기준으로 최적화 데이터 계산 (현재는 사용하지 않지만, 이후 분석용으로 유지)
        let (_optimized_speed, _optimized_viewangle) = strafe_optimize(&frames[i]);

        // onground 동작부
        if frames[i].onground {
            pm_friction(&frames[i], map_data);

            // 연속 동작 중 onground 일 때 fog 증가
            if is_sequenced {
                frames_on_ground = frames_on_ground.saturating_add(1);
                last_segment_index = i;

                // fog 가 10을 초과하면 이전까지를 하나의 기술로 확정
                if frames_on_ground > 10 {
                    if let Some(start) = current_segment_start {
                        let end = last_segment_index.saturating_sub(1).max(start);
                        segments.push(JumpSegment {
                            start_index: start,
                            end_index: end,
                            frames: &frames[start..=end],
                        });
                    }
                    // 시퀀스 초기화
                    is_sequenced = false;
                    frames_on_ground = 0;
                    movements.clear();
                    current_segment_start = None;
                }
            }

            i += 1;
            continue;
        } else {
            // 공중에 있는 동안에는 fog 리셋
            frames_on_ground = 0;
        }

        // 여기부터는 공중 상태에서, 바로 직전(on-ground) 프레임들을 기준으로
        // duck / jump 입력을 감지한다.

        let mut started_this_frame = false;

        // duck 감지 (i-2 가 안전한지 체크)
        if i >= 2 && frames[i - 2].onground
            && frames[i - 2]
                .command
                .iter()
                .any(|cmd| cmd == "+duck")
        {
            if !is_sequenced {
                is_sequenced = true;
                // duck 이 실제로 눌리기 이전 지상 가속 구간까지 포함시키기 위해
                // i-2 이전으로 back-scan 한다.
                let mut start = i - 2;
                // 최대 15프레임(약 0.25초 내외)까지 뒤로 보면서,
                // 지상 상태(on-ground)가 유지되는 범위를 기술 시작 구간으로 본다.
                let mut back = 1usize;
                while back <= 15 && start >= back {
                    let idx = start - back;
                    if !frames[idx].onground {
                        break;
                    }
                    start = idx;
                    back += 1;
                }
                current_segment_start = Some(start);
            } else if let Some(start) = current_segment_start {
                // 이미 시퀀스가 진행 중인 상태에서 추가 duck 이 감지되면,
                // 필요할 경우 더 앞쪽 지상 가속 프레임을 포함시킬 수 있도록 보정
                let mut new_start = (i - 2).min(start);
                let mut back = 1usize;
                while back <= 15 && new_start >= back {
                    let idx = new_start - back;
                    if !frames[idx].onground {
                        break;
                    }
                    new_start = idx;
                    back += 1;
                }
                current_segment_start = Some(new_start);
            }
            movements.push(MovementData {
                movement_type: 0,
                frame: frames[i - 2].frame,
            });
            last_segment_index = i;
            started_this_frame = true;
        }

        // jump 감지 (i-1 이 안전한지 체크)
        if i >= 1 && frames[i - 1].onground
            && frames[i - 1]
                .command
                .iter()
                .any(|cmd| cmd == "+jump")
        {
            if !is_sequenced {
                is_sequenced = true;
                // 점프 이전 지상 가속 구간까지 포함시키기 위해
                // i-1 이전으로 back-scan 한다.
                let mut start = i - 1;
                let mut back = 1usize;
                while back <= 15 && start >= back {
                    let idx = start - back;
                    if !frames[idx].onground {
                        break;
                    }
                    start = idx;
                    back += 1;
                }
                current_segment_start = Some(start);
            } else if let Some(start) = current_segment_start {
                // 기존 시작 인덱스보다 더 앞선 프레임을 시작점으로 삼을 수 있으면,
                // 마찬가지로 그 앞의 지상 가속 구간까지 포함되도록 보정
                let mut new_start = (i - 1).min(start);
                let mut back = 1usize;
                while back <= 15 && new_start >= back {
                    let idx = new_start - back;
                    if !frames[idx].onground {
                        break;
                    }
                    new_start = idx;
                    back += 1;
                }
                current_segment_start = Some(new_start);
            }

            movements.push(MovementData {
                movement_type: 1,
                frame: frames[i - 1].frame,
            });
            last_segment_index = i;
            started_this_frame = true;
        }

        // 시퀀스 중인데 이번 프레임에 새로운 동작이 감지되지 않았다면,
        // 그래도 마지막 인덱스를 갱신해서 구간을 늘려준다.
        if is_sequenced && !started_this_frame {
            last_segment_index = i;
        }

        i += 1;
    }

    // 루프가 끝났는데 아직 열린 시퀀스가 있다면, 마지막까지를 하나의 기술로 처리
    if is_sequenced {
        if let Some(start) = current_segment_start {
            let end = last_segment_index.max(start);
            segments.push(JumpSegment {
                start_index: start,
                end_index: end,
                frames: &frames[start..=end],
            });
        }
    }

    segments
}

