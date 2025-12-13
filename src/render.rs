use crate::bspfile::BspData;
use crate::demo::*;
use crate::analyze::{JumpSegment, angle_vectors};

use plotters::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use gif::{Encoder, Frame, Repeat};

/// 단순 맵 단면도 (기존 기능 유지)
pub fn render_slice_image(bsp: &BspData, center_z: f32, output: &str) {
    // z 범위
    let z_min = center_z - 32.0;
    let z_max = center_z + 32.0;

    // 캔버스
    let root = BitMapBackend::new(output, (2096, 2096)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let (min_x, max_x, min_y, max_y) = bsp.vertexes.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(min_x, max_x, min_y, max_y), v| {
            (
                min_x.min(v.point[0]),
                max_x.max(v.point[0]),
                min_y.min(v.point[1]),
                max_y.max(v.point[1]),
            )
        },
    );

    // 좌표 그리드 추가
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(
            format!("BSP Slice from Z = {:.1} to {:.1}", z_min, z_max),
            ("sans-serif", 20),
        )
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    // 맵의 face 면을 직접 그리기
    for face in &bsp.faces {
        let mut points = Vec::new();

        for i in 0..(face.numedges as usize) {
            let surfedge_index = bsp.surfedges[(face.firstedge as usize) + i];
            let (start, end) = if surfedge_index >= 0 {
                let edge = &bsp.edges[surfedge_index as usize];
                (edge.v[0], edge.v[1])
            } else {
                let edge = &bsp.edges[(-surfedge_index) as usize];
                (edge.v[1], edge.v[0])
            };

            let v0 = bsp.vertexes[start as usize].point;
            let v1 = bsp.vertexes[end as usize].point;

            if (v0[2] < z_min && v1[2] > z_max) || (v1[2] < z_min && v0[2] > z_max) {
                continue;
            }

            if (v0[2] < z_min && v1[2] > z_min) || (v1[2] < z_min && v0[2] > z_min) {
                let t = (z_min - v0[2]) / (v1[2] - v0[2]);
                points.push([
                    v0[0] + t * (v1[0] - v0[0]),
                    v0[1] + t * (v1[1] - v0[1]),
                ]);
            }

            if (v0[2] < z_max && v1[2] > z_max) || (v1[2] < z_max && v0[2] > z_max) {
                let t = (z_max - v0[2]) / (v1[2] - v0[2]);
                points.push([
                    v0[0] + t * (v1[0] - v0[0]),
                    v0[1] + t * (v1[1] - v0[1]),
                ]);
            }
        }

        if points.len() == 2 {
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(points[0][0], points[0][1]), (points[1][0], points[1][1])],
                    &BLACK,
                )))
                .unwrap();
        }
    }

    root.present().unwrap();
}

/// 점프 세그먼트 단면도 + 플레이어/벡터/경로를 그리는 함수
/// - 플레이어: 32x32 정사각형
/// - 중앙: origin 점
/// - wishvel / 진행방향: origin 에서 나가는 선
/// - 세그먼트 전체 경로를 포함하는 영역만 crop 해서 그림
pub fn render_jump_cross_section(
    bsp: &BspData,
    segment: &JumpSegment,
    output: &str,
) {
    // 값 체크
    if segment.frames.is_empty() {
        return;
    }

    // 플레이어 경로의 bounding box 계산
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for f in segment.frames.iter() {
        min_x = min_x.min(f.simorg.x);
        max_x = max_x.max(f.simorg.x);
        min_y = min_y.min(f.simorg.y);
        max_y = max_y.max(f.simorg.y);
        min_z = min_z.min(f.simorg.z);
        max_z = max_z.max(f.simorg.z);
    }

    // 여유 공간 (crop margin)
    let margin_xy = 128.0;
    let center_z = (min_z + max_z) * 0.5;
    let z_min = center_z - 60.0;
    let z_max = center_z + 60.0;

    let mut x_min_view = min_x - margin_xy;
    let mut x_max_view = max_x + margin_xy;
    let mut y_min_view = min_y - margin_xy;
    let mut y_max_view = max_y + margin_xy;

    // degenerate case 방지
    if (x_max_view - x_min_view).abs() < 1.0 {
        x_min_view -= 16.0;
        x_max_view += 16.0;
    }
    if (y_max_view - y_min_view).abs() < 1.0 {
        y_min_view -= 16.0;
        y_max_view += 16.0;
    }

    // 캔버스 생성
    let root = BitMapBackend::new(output, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // 차트 생성
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Jump Cross Section", ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min_view..x_max_view, y_min_view..y_max_view)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    // 맵 단면(face) 그리기 (위의 render_slice_image 와 동일 로직)
    for face in &bsp.faces {
        let mut points = Vec::new();

        for i in 0..(face.numedges as usize) {
            let surfedge_index = bsp.surfedges[(face.firstedge as usize) + i];
            let (start, end) = if surfedge_index >= 0 {
                let edge = &bsp.edges[surfedge_index as usize];
                (edge.v[0], edge.v[1])
            } else {
                let edge = &bsp.edges[(-surfedge_index) as usize];
                (edge.v[1], edge.v[0])
            };

            let v0 = bsp.vertexes[start as usize].point;
            let v1 = bsp.vertexes[end as usize].point;

            if (v0[2] < z_min && v1[2] > z_max) || (v1[2] < z_min && v0[2] > z_max) {
                continue;
            }

            if (v0[2] < z_min && v1[2] > z_min) || (v1[2] < z_min && v0[2] > z_min) {
                let t = (z_min - v0[2]) / (v1[2] - v0[2]);
                points.push([
                    v0[0] + t * (v1[0] - v0[0]),
                    v0[1] + t * (v1[1] - v0[1]),
                ]);
            }

            if (v0[2] < z_max && v1[2] > z_max) || (v1[2] < z_max && v0[2] > z_max) {
                let t = (z_max - v0[2]) / (v1[2] - v0[2]);
                points.push([
                    v0[0] + t * (v1[0] - v0[0]),
                    v0[1] + t * (v1[1] - v0[1]),
                ]);
            }
        }

        if points.len() == 2 {
            // 선 그리기
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(points[0][0], points[0][1]), (points[1][0], points[1][1])],
                    &BLACK,
                )))
                .unwrap();
        }
    }

    // 플레이어 경로(line) 그리기
    chart
        .draw_series(LineSeries::new(
            segment
                .frames
                .iter()
                .map(|f| (f.simorg.x, f.simorg.y)),
            &RED,
        ))
        .unwrap();

    // 시작 프레임 기준으로 플레이어 박스 / origin / 벡터 그리기
    let start_frame = &segment.frames[0];
    let px = start_frame.simorg.x;
    let py = start_frame.simorg.y;

    // 플레이어 32x32 정사각형
    let half_size = 16.0;
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(px - half_size, py - half_size), (px + half_size, py + half_size)],
            ShapeStyle {
                color: BLUE.mix(0.5),
                filled: false,
                stroke_width: 2,
            },
        )))
        .unwrap();

    // origin 점
    chart
        .draw_series(std::iter::once(Circle::new(
            (px, py),
            3,
            ShapeStyle::from(&BLACK).filled(),
        )))
        .unwrap();

    // wishvel / 진행 방향 벡터 계산
    let (forward, right, up) = angle_vectors(&start_frame.viewangle);

    let wishvel = Vector3 {
        x: forward.x * start_frame.forwardmove
            + right.x * start_frame.sidemove
            + up.x * start_frame.upmove,
        y: forward.y * start_frame.forwardmove
            + right.y * start_frame.sidemove
            + up.y * start_frame.upmove,
        z: 0.0,
    };

    let wish_dir = wishvel.normalize();
    let wish_len = 64.0;

    // wishvel 선 (파란색)
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (px, py),
                (px + wish_dir.x * wish_len, py + wish_dir.y * wish_len),
            ],
            &BLUE,
        )))
        .unwrap();

    // 실제 진행 방향 (simvel) 선 (초록색)
    let mut move_dir = Vector3 {
        x: start_frame.simvel.x,
        y: start_frame.simvel.y,
        z: 0.0,
    };
    let len = move_dir.length();
    if len != 0.0 {
        move_dir.x /= len;
        move_dir.y /= len;
    }

    let move_len = 64.0;
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (px, py),
                (px + move_dir.x * move_len, py + move_dir.y * move_len),
            ],
            &GREEN,
        )))
        .unwrap();

    root.present().unwrap();
}

/// 점프 세그먼트 전체 궤적을 프레임 단위로 렌더링하여 하나의 GIF 로 저장
/// - 매 프레임마다:
///   - 플레이어 위치(32x32 박스 + origin 점)
///   - 이동 방향(simvel, 파란색)
///   - 시야/wish 방향(노란색)
///   - 지금까지의 궤적(trace, 빨간색)
///   을 갱신해서 그린다.
/// - 배경 맵은 점프 전체 궤적을 모두 포함하는 영역으로 고정된다.
pub fn render_jump_gif(
    bsp: &BspData,
    segment: &JumpSegment,
    output_gif: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if segment.frames.is_empty() {
        return Ok(());
    }

    // 세그먼트 전체 궤적의 XY/Z 범위를 계산
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    for f in segment.frames.iter() {
        min_x = min_x.min(f.simorg.x);
        max_x = max_x.max(f.simorg.x);
        min_y = min_y.min(f.simorg.y);
        max_y = max_y.max(f.simorg.y);
        min_z = min_z.min(f.simorg.z);
        max_z = max_z.max(f.simorg.z);
    }

    // Z는 중앙 기준 ±60, XY는 전체 궤적 + 여유 100 유닛
    let center_z = (min_z + max_z) * 0.5;
    let z_min = center_z - 60.0;
    let z_max = center_z + 60.0;

    let margin_xy = 200.0;
    let x_min_view = min_x - margin_xy;
    let x_max_view = max_x + margin_xy;
    let y_min_view = min_y - margin_xy;
    let y_max_view = max_y + margin_xy;

    // 월드 영역의 가로/세로 비율에 맞춰 GIF 캔버스의 width/height 를 결정
    // (픽셀 수는 대략 base_px 에 맞추되, 비율은 강제로 바꾸지 않음)
    let base_px: u32 = 600;
    let world_w = (x_max_view - x_min_view).abs().max(1.0);
    let world_h = (y_max_view - y_min_view).abs().max(1.0);
    let (width, height) = if world_w >= world_h {
        let h = ((base_px as f32) * world_h / world_w).max(1.0) as u32;
        (base_px, h)
    } else {
        let w = ((base_px as f32) * world_w / world_h).max(1.0) as u32;
        (w, base_px)
    };

    println!("[render_jump_gif] start, frames in segment = {}", segment.frames.len());

    let file = File::create(output_gif)?;
    let mut encoder = Encoder::new(BufWriter::new(file), width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    // 프레임을 하나도 생략하지 않고 모든 프레임을 사용
    let total_frames = segment.frames.len();
    let step: usize = 1;

    // 이전 프레임까지의 궤적을 누적해서 사용 (0..=idx 구간)
    let mut gif_frame_count: usize = 0;
    for fi in 0..total_frames {
        let frame = &segment.frames[fi];
        let mut buffer = vec![0u8; (width * height * 3) as usize];

        {
            let root =
                BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                    .into_drawing_area();
            root.fill(&WHITE)?;

            // 배경 뷰포트는 세그먼트 전체 궤적을 모두 포함하는 고정 영역
            let px = frame.simorg.x;
            let py = frame.simorg.y;

            let mut chart = ChartBuilder::on(&root)
                .margin(5)
                .x_label_area_size(20)
                .y_label_area_size(20)
                .build_cartesian_2d(x_min_view..x_max_view, y_min_view..y_max_view)?;

            // 배경 맵 단면(face) 그리기 (render_jump_cross_section 와 동일 로직)
            for face in &bsp.faces {
                let mut points = Vec::new();

                for i in 0..(face.numedges as usize) {
                    let surfedge_index = bsp.surfedges[(face.firstedge as usize) + i];
                    let (start, end) = if surfedge_index >= 0 {
                        let edge = &bsp.edges[surfedge_index as usize];
                        (edge.v[0], edge.v[1])
                    } else {
                        let edge = &bsp.edges[(-surfedge_index) as usize];
                        (edge.v[1], edge.v[0])
                    };

                    let v0 = bsp.vertexes[start as usize].point;
                    let v1 = bsp.vertexes[end as usize].point;

                    if (v0[2] < z_min && v1[2] > z_max)
                        || (v1[2] < z_min && v0[2] > z_max)
                    {
                        continue;
                    }

                    if (v0[2] < z_min && v1[2] > z_min)
                        || (v1[2] < z_min && v0[2] > z_min)
                    {
                        let t = (z_min - v0[2]) / (v1[2] - v0[2]);
                        points.push([
                            v0[0] + t * (v1[0] - v0[0]),
                            v0[1] + t * (v1[1] - v0[1]),
                        ]);
                    }

                    if (v0[2] < z_max && v1[2] > z_max)
                        || (v1[2] < z_max && v0[2] > z_max)
                    {
                        let t = (z_max - v0[2]) / (v1[2] - v0[2]);
                        points.push([
                            v0[0] + t * (v1[0] - v0[0]),
                            v0[1] + t * (v1[1] - v0[1]),
                        ]);
                    }
                }

                if points.len() == 2 {
                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![(points[0][0], points[0][1]), (points[1][0], points[1][1])],
                        &BLACK,
                    )))?;
                }
            }


            // 지금까지의 궤적(trace) 그리기 (0..=fi)
            chart.draw_series(LineSeries::new(
                segment.frames[..=fi]
                    .iter()
                    .map(|f| (f.simorg.x, f.simorg.y)),
                &RED,
            ))?;

            // 플레이어 32x32 박스 + origin
            let half_size = 16.0;
            chart.draw_series(std::iter::once(Rectangle::new(
                [(px - half_size, py - half_size), (px + half_size, py + half_size)],
                ShapeStyle {
                    color: GREEN.mix(0.5),
                    filled: false,
                    stroke_width: 2,
                },
            )))?;

            chart.draw_series(std::iter::once(Circle::new(
                (px, py),
                3,
                ShapeStyle::from(&BLACK).filled(),
            )))?;

            // wish 방향 (노란색)
            let (forward, right, up) = angle_vectors(&frame.viewangle);
            let wishvel = Vector3 {
                x: forward.x * frame.forwardmove
                    + right.x * frame.sidemove
                    + up.x * frame.upmove,
                y: forward.y * frame.forwardmove
                    + right.y * frame.sidemove
                    + up.y * frame.upmove,
                z: 0.0,
            };
            let wish_dir = wishvel.normalize();
            let wish_len = 64.0;

            chart.draw_series(std::iter::once(PathElement::new(
                vec![
                    (px, py),
                    (px + wish_dir.x * wish_len, py + wish_dir.y * wish_len),
                ],
                &BLACK,
            )))?;

            // 이동 방향(simvel, 파란색)
            let mut move_dir = Vector3 {
                x: frame.simvel.x,
                y: frame.simvel.y,
                z: 0.0,
            };
            let len = move_dir.length();
            if len != 0.0 {
                move_dir.x /= len;
                move_dir.y /= len;
            }
            let move_len = 64.0;

            chart.draw_series(std::iter::once(PathElement::new(
                vec![
                    (px, py),
                    (px + move_dir.x * move_len, py + move_dir.y * move_len),
                ],
                &BLUE,
            )))?;

            root.present()?;
        }

        // GIF 프레임 추가 (delay: frametime * step 기반, 1/100초 단위)
        let mut gif_frame =
            Frame::from_rgb(width as u16, height as u16, &buffer);
        let delay_cs = (frame.frametime * step as f32 * 100.0).max(1.0) as u16;
        gif_frame.delay = delay_cs;
        encoder.write_frame(&gif_frame)?;

        gif_frame_count += 1;
        // 진행 상황 로그 (테스트용)
        println!(
            "[render_jump_gif] rendered GIF frame {} (source frame {}/{}, step = {}).",
            gif_frame_count,
            fi + 1,
            total_frames,
            step
        );
    }

    Ok(())
}