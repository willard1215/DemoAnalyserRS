use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::mem::size_of;
use byteorder::{LittleEndian, ReadBytesExt};

//bsp lump 구조체
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Lump {
    pub fileofs: i32,
    pub filelen: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct _DHeader {
    pub version: i32,
    pub lumps: [Lump; 15],
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

pub struct BspData {
    pub _models: Vec<DModel>,
    pub vertexes: Vec<DVertex>,
    pub _planes: Vec<DPlane>,
    pub faces: Vec<DFace>,
    pub edges: Vec<DEdge>,
    pub surfedges: Vec<i32>,
}

pub fn load_bsp_file<P: AsRef<Path>>(path: P) -> std::io::Result<BspData> {
    let mut file = File::open(path)?;

    let mut version_buf = [0u8; 4];
    file.read_exact(&mut version_buf)?;
    let version = i32::from_le_bytes(version_buf);
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

    let _models = read_lump::<DModel>(&mut file, &lumps[14])?;
    let vertexes = read_lump::<DVertex>(&mut file, &lumps[3])?;
    let _planes = read_lump::<DPlane>(&mut file, &lumps[1])?;
    let faces = read_lump::<DFace>(&mut file, &lumps[7])?;
    let edges = read_lump::<DEdge>(&mut file, &lumps[12])?;

    let surfedges = {
        let size = size_of::<i32>();
        let count = lumps[13].filelen as usize / size;
        let mut vec = Vec::with_capacity(count);
        file.seek(SeekFrom::Start(lumps[13].fileofs as u64))?;
        for _ in 0..count {
            vec.push(file.read_i32::<LittleEndian>()?);
        }
        vec
    };

    Ok(BspData {
        _models,
        vertexes,
        _planes,
        faces,
        edges,
        surfedges,
    })
}

