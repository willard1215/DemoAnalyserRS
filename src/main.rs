mod bspfile;
mod render;
mod demo;
mod analyze;

use bspfile::load_bsp_file;
// use bspfile::{load_bsp_file};
use demo::{parse};
// use render::*;
use analyze::*;

use render::*;


fn main() {
    let demo: &str = "274_dcj_Desu.dem";
    // let demo: &str = "test.dem";

    match parse(&demo) {
        Ok(parsed_data) => {
            let data: Vec<JumpSegment>= extract_jump_segments(&parsed_data);
            if let Some(segment) = data.first() {
                println!("Jump segment: {} to {}", segment.start_index, segment.end_index);
                // for frame in &segment.frames {
                //     println!("{}",frame.msec)
                // };
            } else {
                println!("No jump segments found.");
}

        }
        Err(e) => {
            eprintln!("Failed to parse demo: {}", e);
        }
    }

    // let map = load_bsp_file("cs_assault.bsp").unwrap();
    // render_slice_image(&map,20.0,"output.bmp");
}