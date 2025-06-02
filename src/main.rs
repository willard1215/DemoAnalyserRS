mod bspfile; //bsp 구조체 파싱모듈
mod render; //렌더링 모듈
mod demo; //데모 파싱모듈
mod analyze; //데모 분석모듈

use bspfile::load_bsp_file;
use demo::{parse};
use analyze::*;
use render::{render_slice_image};


fn main() {
    let map_name = "./test/kz_longjumps2.bsp";
    let map_data = match load_bsp_file(map_name) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read map data: {}", e);
            return;
        }
    }; 
    render_slice_image(&map_data, -100.0, "./test/test.png");
    let demo: &str = "./test/274_dcj_Desu.dem";

    match parse(&demo) {
        Ok(parsed_data) => {
            let data: Vec<JumpSegment>= extract_jump_segments(&parsed_data);
            if let Some(segment) = data.first() {
                println!("Jump segment: {} to {}", segment.start_index, segment.end_index);
            } else {
                println!("No jump segments found.");
            }

        }
        Err(e) => {
            eprintln!("Failed to parse demo: {}", e);
        }
    }
   
}