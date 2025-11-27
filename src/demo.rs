use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn dot(&self, other: Vector3) -> f32 {
        return self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(&self) -> f32 {
        return (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_3d(&self) -> f32{
        return (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vector3 {
        let length = (self.x * self.x + self.y * self.y).sqrt();
        if length == 0.0 {
            return Vector3 {x:0.0, y: 0.0, z: 0.0};
        }

        let normalized = Vector3 { x: (self.x / length), y: (self.y / length), z: (self.z / length) };

        return normalized
    }
}

#[derive(Debug, Clone)]
pub struct DemoFrame {
    pub frame: i32,
    pub time: f32,
    pub vieworg: Vector3,
    pub viewangle: Vector3,
    pub frametime: f32,
    pub onground: bool,
    pub simvel: Vector3,
    pub simorg: Vector3,
    pub viewheight: Vector3,
    pub msec: u8,
    pub gravity: f32,
    pub accelerate: f32,
    pub airaccelerate: f32,
    pub friction: f32,
    pub edgefriction: f32,
    pub maxvelocity: f32,
    pub command: Vec<String>,
    pub forwardmove: f32,
    pub sidemove: f32,
    pub upmove: f32,
    pub forward: Vector3,
    pub right: Vector3,
    pub up: Vector3,
}


pub fn parse(path: &str) -> io::Result<Vec<DemoFrame>> {
    
    fn read_fixed_string<R: Read>(reader: &mut R, size: usize) -> io::Result<String> {
        let mut buf = vec![0u8; size];
        reader.read_exact(&mut buf)?;
        let string = buf.split(|&b| b == 0)
                        .next()
                        .unwrap_or(&[])
                        .iter()
                        .map(|&c| c as char)
                        .collect::<String>();
        Ok(string)
    }

    let mut file = File::open(path)?;

    // HLDEMO 매직스트링
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic[..6] != b"HLDEMO" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Demo Header"));
    }

    let _demo_version = file.read_i32::<LittleEndian>()?;
    if _demo_version != 5 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a cs 1.6 Demo"));
    }

    let _network_version = file.read_i32::<LittleEndian>()?;
    let _map_name = read_fixed_string(&mut file, 260)?;
    let _game_dll = read_fixed_string(&mut file, 260)?;
    let _crc = file.read_i32::<LittleEndian>()?;
    let dir_offset = file.read_i32::<LittleEndian>()?;

    // 오프셋으로 이동 - 데모 마지막 부분
    file.seek(SeekFrom::Start(dir_offset as u64))?;
    let entry_count = file.read_i32::<LittleEndian>()?;

    let mut entry_offsets = Vec::new();
    for _i in 0..entry_count {
        let _entry_type = file.read_i32::<LittleEndian>()?;
        let _title = read_fixed_string(&mut file, 64)?;
        let _flags = file.read_i32::<LittleEndian>()?;
        let _play = file.read_i32::<LittleEndian>()?;
        let _time = file.read_f32::<LittleEndian>()?;
        let _frames = file.read_i32::<LittleEndian>()?;
        let offset = file.read_i32::<LittleEndian>()?;
        let _length = file.read_i32::<LittleEndian>()?;
        entry_offsets.push(offset);
    }

    // 실제 게임 데이터 부로 포인터 이동
    let offset = entry_offsets[1] as u64;
    file.seek(SeekFrom::Start(offset))?;
    
    let mut frames: Vec<DemoFrame> = Vec::new();

    let mut commands_by_frame: HashMap<i32, Vec<String>> = HashMap::new();
    
    loop {
        let demo_type = match file.read_u8() {
            Ok(v) => v,
            Err(_) => break,
        };

        let time = file.read_f32::<LittleEndian>()?;
        let frame = file.read_i32::<LittleEndian>()?;

        //네트워크 메세지 - 실제 서버 통신내용
        if demo_type == 1 {
            let _timestamp = file.read_f32::<LittleEndian>()?;

            let vieworg = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let viewangle = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let forward = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let right = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let up = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let frametime = file.read_f32::<LittleEndian>()?;
            let _time = file.read_f32::<LittleEndian>()?;
            let _intermission = file.read_i32::<LittleEndian>()?;
            let _puased = file.read_i32::<LittleEndian>()?;
            let _spectator = file.read_i32::<LittleEndian>()?;
            let onground = file.read_i32::<LittleEndian>()? != 0;
            
            let _waterlevel = file.read_i32::<LittleEndian>()?;

            let simvel = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let simorg = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let viewheight = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let _idealpitch = file.read_f32::<LittleEndian>()?;

            let _clviewangles = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };
            
            let _health = file.read_f32::<LittleEndian>()?;

            let _crosshairangle = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let _viewsize = file.read_i32::<LittleEndian>()?;

            let _punchangle = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let _maxclients = file.read_i32::<LittleEndian>()?;
            let _viewentity = file.read_i32::<LittleEndian>()?;
            let _playernum = file.read_i32::<LittleEndian>()?;
            let _maxentities = file.read_i32::<LittleEndian>()?;
            let _demoplayback = file.read_i32::<LittleEndian>()?;
            let _hardware = file.read_i32::<LittleEndian>()?;
            let _smoothing = file.read_i32::<LittleEndian>()?;
            let _ptrcmd = file.read_i32::<LittleEndian>()?;
            let _ptrmovevars = file.read_i32::<LittleEndian>()?;
            let _viewport_x = file.read_i32::<LittleEndian>()?;
            let _viewport_y = file.read_i32::<LittleEndian>()?;
            let _resolutionwidth = file.read_i32::<LittleEndian>()?;
            let _resolutionheight = file.read_i32::<LittleEndian>()?;
            let _nextview = file.read_i32::<LittleEndian>()?;
            let _onlyclientdraw = file.read_i32::<LittleEndian>()?;
            let _lerpmsec = file.read_u16::<LittleEndian>()?;
            let msec = file.read_u8()?;
            let _align1 = file.read_u8()?;
            
            let _viewangles2 = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let forwardmove = file.read_f32::<LittleEndian>()?;
            let sidemove = file.read_f32::<LittleEndian>()?;
            let upmove = file.read_f32::<LittleEndian>()?;
            let _lightlevel = file.read_u8()?;
            let _align2 = file.read_u8()?;
            let _buttons = file.read_u16::<LittleEndian>()?;
            let _impulse = file.read_u8()?;
            let _weaponselect = file.read_u8()?;
            let _align3 = file.read_u8()?;
            let _align4 = file.read_u8()?;
            let _impactindex = file.read_i32::<LittleEndian>()?;
            
            let _impactposition = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let gravity = file.read_f32::<LittleEndian>()?;
            let _stopspeed = file.read_f32::<LittleEndian>()?;
            let _maxspeed = file.read_f32::<LittleEndian>()?;
            let _spectatormaxspeed = file.read_f32::<LittleEndian>()?;
            let accelerate = file.read_f32::<LittleEndian>()?;
            let airaccelerate = file.read_f32::<LittleEndian>()?;
            let _wateraccelerate = file.read_f32::<LittleEndian>()?;
            let friction = file.read_f32::<LittleEndian>()?;
            let edgefriction = file.read_f32::<LittleEndian>()?;
            let _waterfriction = file.read_f32::<LittleEndian>()?;
            let _entgravity = file.read_f32::<LittleEndian>()?;
            let _bounce = file.read_f32::<LittleEndian>()?;
            let _stepsize = file.read_f32::<LittleEndian>()?;
            let maxvelocitiy = file.read_f32::<LittleEndian>()?;
            let _zmax = file.read_f32::<LittleEndian>()?;
            let _waveheight = file.read_f32::<LittleEndian>()?;
            let _footsteps = file.read_i32::<LittleEndian>()?;
            let _skyname = read_fixed_string(&mut file, 32)?;
            let _rollangle = file.read_f32::<LittleEndian>()?;
            let _rollspeed = file.read_f32::<LittleEndian>()?;
            let _skycolor_r = file.read_f32::<LittleEndian>()?;
            let _skycolor_g = file.read_f32::<LittleEndian>()?;
            let _skycolor_b = file.read_f32::<LittleEndian>()?;
            
            let _sky_vector = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let _view = Vector3 {
                x: file.read_f32::<LittleEndian>()?,
                y: file.read_f32::<LittleEndian>()?,
                z: file.read_f32::<LittleEndian>()?,
            };

            let _viewmodel = file.read_i32::<LittleEndian>()?;
            let _incoming_sequence = file.read_i32::<LittleEndian>()?;
            let _incoming_ack = file.read_i32::<LittleEndian>()?;
            let _incoming_reliable_ack = file.read_i32::<LittleEndian>()?;
            let _incoming_reliable_seq = file.read_i32::<LittleEndian>()?;
            let _outgoing_seq = file.read_i32::<LittleEndian>()?;
            let _reliable_seq = file.read_i32::<LittleEndian>()?;
            let _lastreliable_sez = file.read_i32::<LittleEndian>()?;
            let msglength = file.read_i32::<LittleEndian>()? as usize;
            let mut buffer = vec![0u8; msglength];

            file.read_exact(&mut buffer)?;
            let _chatmessage = buffer.iter()
                .map(|&b| b as char)
                .take_while(|&c| c != '\0')
                .collect::<String>();
            

            let joined_cmds = commands_by_frame.remove(&frame)
                .map(|cmds| cmds.iter().map(|c| format!("{}", c)).collect::<Vec<_>>())
                .unwrap_or_default();
            
            // if !joined_cmds.is_empty() {
            //     println!("{:#?}",joined_cmds);
            // }
            

            frames.push(DemoFrame { 
                frame: (frame), 
                time: (time), 
                vieworg: (vieworg),
                viewangle: (viewangle), 
                frametime: (frametime), 
                onground: (onground), 
                simvel: (simvel), 
                simorg: (simorg), 
                viewheight: (viewheight),
                msec: (msec), 
                gravity: (gravity), 
                accelerate: (accelerate), 
                airaccelerate: (airaccelerate), 
                friction: (friction),
                edgefriction: (edgefriction),
                maxvelocity: (maxvelocitiy), 
                command: (joined_cmds),
                forwardmove: (forwardmove),
                sidemove: (sidemove),
                upmove: (upmove),
                forward: (forward),
                right: (right),
                up: (up),
            });
        }
        //파싱시작부
        else if demo_type == 2 {
            // println!("Demo Started")
        }
        //커멘드
        else if demo_type == 3 {
            let cmd = read_fixed_string(&mut file, 64)?;
            commands_by_frame.entry(frame).or_default().push(cmd);
        }
        //클라이언트 내부 지표 - 스킵
        else if demo_type == 4{
            file.seek(SeekFrom::Current(32))?;
        }
        //EOF 플래그
        else if demo_type == 5 {
            // println!("EOF.");
            break;
        }
        //이벤트 상호작용 데이터 - 스킵
        else if demo_type == 6 {
            file.seek(SeekFrom::Current(84))?;
        }
        //무기 에니메이션 데이터 - 스킵
        else if demo_type == 7 {
            file.seek(SeekFrom::Current(8))?;
        }
        //사운드 데이터
        else if demo_type == 8 {
            let _channel = file.read_i32::<LittleEndian>()?;
            let sample_len = file.read_i32::<LittleEndian>()? as usize;
            let mut buffer = vec![0u8; sample_len];
            file.read_exact(&mut buffer)?;
            let _sample = String::from_utf8_lossy(&buffer).trim_end_matches('\0').to_string();
            let _attenuation = file.read_f32::<LittleEndian>()?;
            let _volume = file.read_f32::<LittleEndian>()?;
            let _flags = file.read_i32::<LittleEndian>()?;
            let _pitch = file.read_i32::<LittleEndian>()?;
        }
        else if demo_type == 9 {
            let buffer_length = file.read_i32::<LittleEndian>()? as usize;
            let mut buffer = vec![0u8; buffer_length];
            file.read_exact(&mut buffer)?;
        } else {
            println!("Skipping unhandled segment type: {}", demo_type);
            file.seek(SeekFrom::Current(4))?;
        }
    }

    Ok(frames)
}
