use crate::bspfile::{BspData};
use crate::demo::*;

use plotters::prelude::*;

pub fn render_slice_image(bsp: &BspData, center_z: f32, output: &str) {
    // z 범위
    let z_min = center_z - 60.0;
    let z_max = center_z + 60.0;

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
        .caption(format!("BSP Slice from Z = {:.1} to {:.1}", z_min, z_max), ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    //맵의 face면
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
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(points[0][0], points[0][1]), (points[1][0], points[1][1])],
                &BLACK,
            ))).unwrap();
        }
    }

    root.present().unwrap();
}