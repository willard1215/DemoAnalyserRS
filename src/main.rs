mod bspfile; //bsp 구조체 파싱모듈
mod render; //렌더링 모듈
mod demo; //데모 파싱모듈
mod analyze; //데모 분석모듈

use bspfile::load_bsp_file;
use demo::parse;
use analyze::*;
use render::{render_slice_image, render_jump_cross_section, render_jump_gif};

fn main() {
    // 1. 맵 로드
    let map_name = "./test/maps/kz_longjumps2.bsp";
    let map_data = match load_bsp_file(map_name) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read map data: {}", e);
            return;
        }
    };

    // 2. 데모 파싱
    let demo: &str = "./test/274_dcj_Desu.dem";
    let parsed = match parse(demo) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse demo: {}", e);
            return;
        }
    };

    // 3. 점프 세그먼트 추출
    let segments: Vec<JumpSegment> = extract_jump_segments(&parsed, &map_data);
    println!("Detected jump segments: {}", segments.len());

    // 4. 첫 번째 세그먼트를 대상으로 PNG + GIF 테스트 렌더링
    if let Some(first) = segments.first() {
        println!("Rendering jump cross-section PNG...");
        // 단면 PNG
        render_jump_cross_section(&map_data, first, "./test/jump_segment.png");
        println!("Done: jump_segment.png");

        println!(
            "Rendering jump GIF (frames = {})...",
            first.frames.len()
        );
        // 전체 궤적 GIF
        if let Err(err) = render_jump_gif(&map_data, first, "./test/jump_segment.gif") {
            eprintln!("Failed to render jump GIF: {}", err);
        }
        println!("Done: jump_segment.gif");
    } else {
        println!("No jump segments found.");
    }

    // 5. 맵 전체 단면도 PNG (디버그용)
    render_slice_image(&map_data, -70.0, "./test/test.png");
}