use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Lump {
    pub fileofs: i32,
    pub filelen: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DModel {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub origin: [f32; 3],
    pub headnode: [i32; 4],
    pub visleafs: i32,
    pub firstface: i32,
    pub numfaces: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DVertex {
    pub point: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DPlane {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DNode {
    pub planenum: i32,
    pub children: [i16; 2],
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub firstface: u16,
    pub numfaces: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DLeaf {
    pub contents: i32,
    pub visofs: i32,
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub firstleafface: u16,
    pub numleaffaces: u16,
    pub firstleafbrush: u16,
    pub numleafbrushes: u16,
    pub ambient_level: [u8; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DFace {
    pub planenum: i16,
    pub side: i16,
    pub firstedge: i32,
    pub numedges: i16,
    pub texinfo: i16,
    pub styles: [u8; 4],
    pub lightofs: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DEdge {
    pub v: [u16; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DTexInfo {
    pub vecs: [[f32; 4]; 2],
    pub miptex: i32,
    pub flags: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DBrush {
    pub firstside: i32,
    pub numsides: i32,
    pub contents: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DBrushSide {
    pub planenum: u16,
    pub texinfo: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DClipNode {
    pub planenum: i32,
    pub children: [i16; 2],
}

pub struct BspData {
    pub entities: String,
    pub planes: Vec<DPlane>,
    pub texinfo: Vec<DTexInfo>,
    pub faces: Vec<DFace>,
    pub nodes: Vec<DNode>,
    pub leafs: Vec<DLeaf>,
    pub leaffaces: Vec<u16>,
    pub leafbrushes: Vec<u16>,
    pub models: Vec<DModel>,
    pub brushes: Vec<DBrush>,
    pub brushsides: Vec<DBrushSide>,
    pub vertexes: Vec<DVertex>,
    pub edges: Vec<DEdge>,
    pub surfedges: Vec<i32>,
    pub visdata: Vec<u8>,
    pub clipnodes: Vec<DClipNode>,
    pub lightmaps: Vec<u8>,
}

pub fn load_bsp_file<P: AsRef<Path>>(path: P) -> std::io::Result<BspData> {
    let mut file = File::open(path)?;

    let version = file.read_i32::<LittleEndian>()?;
    if version != 30 {
        panic!("Unsupported BSP version: {}", version);
    }

    let mut lumps = [Lump { fileofs: 0, filelen: 0 }; 15];
    for lump in lumps.iter_mut() {
        lump.fileofs = file.read_i32::<LittleEndian>()?;
        lump.filelen = file.read_i32::<LittleEndian>()?;
    }

    fn read_lump<T: Copy + Default>(file: &mut File, lump: &Lump) -> std::io::Result<Vec<T>> {
        let size = size_of::<T>();
        let count = lump.filelen as usize / size;
        let mut vec = Vec::with_capacity(count);
        file.seek(SeekFrom::Start(lump.fileofs as u64))?;
        let mut buffer = vec![0u8; lump.filelen as usize];
        file.read_exact(&mut buffer)?;
        for chunk in buffer.chunks_exact(size) {
            let mut arr = [0u8; 64];
            arr[..size].copy_from_slice(chunk);
            let t = unsafe { std::ptr::read_unaligned(arr.as_ptr() as *const T) };
            vec.push(t);
        }
        Ok(vec)
    }

    let entities = {
        file.seek(SeekFrom::Start(lumps[0].fileofs as u64))?;
        let mut buf = vec![0u8; lumps[0].filelen as usize];
        file.read_exact(&mut buf)?;
        String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string()
    };

    let planes = read_lump::<DPlane>(&mut file, &lumps[1])?;
    let texinfo = read_lump::<DTexInfo>(&mut file, &lumps[2])?;
    let vertexes = read_lump::<DVertex>(&mut file, &lumps[3])?;
    let clipnodes = read_lump::<DClipNode>(&mut file, &lumps[4])?;
    let nodes = read_lump::<DNode>(&mut file, &lumps[5])?;
    let brushes = read_lump::<DBrush>(&mut file, &lumps[6])?;
    let faces = read_lump::<DFace>(&mut file, &lumps[7])?;

    let lightmaps = {
        file.seek(SeekFrom::Start(lumps[8].fileofs as u64))?;
        let mut buf = vec![0u8; lumps[8].filelen as usize];
        file.read_exact(&mut buf)?;
        buf
    };

    let brushsides = read_lump::<DBrushSide>(&mut file, &lumps[9])?;
    let leafs = read_lump::<DLeaf>(&mut file, &lumps[10])?;

    let leaffaces = {
        file.seek(SeekFrom::Start(lumps[11].fileofs as u64))?;
        let mut buf = vec![0u8; lumps[11].filelen as usize];
        file.read_exact(&mut buf)?;
        buf.chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect()
    };

    let edges = read_lump::<DEdge>(&mut file, &lumps[12])?;

    let surfedges = {
        file.seek(SeekFrom::Start(lumps[13].fileofs as u64))?;
        let count = lumps[13].filelen as usize / size_of::<i32>();
        (0..count).map(|_| file.read_i32::<LittleEndian>()).collect::<Result<_, _>>()?
    };

    let leafbrushes = {
        file.seek(SeekFrom::Start(lumps[13].fileofs as u64))?;
        let mut buf = vec![0u8; lumps[13].filelen as usize];
        file.read_exact(&mut buf)?;
        buf.chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect()
    };

    let models = read_lump::<DModel>(&mut file, &lumps[14])?;

    let visdata = {
        file.seek(SeekFrom::Start(lumps[4].fileofs as u64))?;
        let mut buf = vec![0u8; lumps[4].filelen as usize];
        file.read_exact(&mut buf)?;
        buf
    };

    Ok(BspData {
        entities,
        planes,
        texinfo,
        faces,
        nodes,
        leafs,
        leaffaces,
        leafbrushes,
        models,
        brushes,
        brushsides,
        vertexes,
        edges,
        surfedges,
        visdata,
        clipnodes,
        lightmaps,
    })
}

