use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use anyhow::{Ok, Result, anyhow};
use bytemuck::{Pod, Zeroable};
use ply_rs::{
    parser::{self},
    ply::{self, Encoding, Header, PropertyType},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    core::quaternion::Quat,
    prelude::{BVH, Vec3f},
    raycast::{Hit, Ray, Raycast, bounds::Bounds3f, primitive::Primitive},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InputSplat {
    pub pos: [f32; 3],
    pub nor: [f32; 3],
    pub dc0: [f32; 3],
    pub sh: [f32; 3 * 15],
    pub opacity: f32,
    pub scale: [f32; 3],
    pub rot: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct Splat {
    pub pos: Vec3f,
    pub nor: Vec3f,
    pub col: Vec3f,
    pub sh: [Vec3f; 15],
    pub opacity: f32,
    pub scale: Vec3f,
    pub rot: Quat,
}

impl Splat {
    pub fn from_input(input: &InputSplat) -> Self {
        Splat {
            // testing
            pos: Vec3f::vec(input.pos),
            nor: Vec3f::vec(input.nor),
            col: Vec3f::vec(input.dc0),
            sh: [Vec3f::vec([0.; 3]); 15],
            opacity: input.opacity,
            scale: Vec3f::vec(input.scale),
            rot: Quat::from_xyzw(input.rot[0], input.rot[1], input.rot[2], input.rot[3]),
        }
    }
}

impl Raycast for Splat {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        self.bounds().raycast(ray)
    }
}

impl Primitive for Splat {
    fn bounds(&self) -> Bounds3f {
        let one = Vec3f::vec([1.; 3]);
        Bounds3f {
            min: self.pos - one,
            max: self.pos + one,
        }
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(*self)
    }
}

/// a collection of splats
pub struct GaussianScene {
    //TODO:build bvh
    pub bvh: BVH,
}

impl GaussianScene {
    pub fn from_ply(path: &str) -> Result<Self> {
        let input_gs = Self::read_ply(path)?;
        let splats: Vec<Splat> = input_gs.par_iter().map(Splat::from_input).collect();

        let mut bvh = BVH::new(splats.len());

        (0..splats.len()).for_each(|i| {
            bvh.push(splats[i]);
        });

        // test other prim limit
        bvh.build(129, true);

        Ok(GaussianScene { bvh })
    }

    pub fn read_ply(path: &str) -> Result<Vec<InputSplat>> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

        let (header, prop_sizes) = Self::read_ply_header(&mut reader)?;
        let stride = prop_sizes.iter().sum();

        let splat_count = header.elements.get("vertex").unwrap().count;
        let mut buf: Vec<u8> = vec![0; stride];
        let mut splats = Vec::with_capacity(splat_count);

        for _ in 0..splat_count {
            reader.read_exact(&mut buf)?;
            let gs: &InputSplat = bytemuck::from_bytes(&buf);
            splats.push(*gs);
        }

        Ok(splats)
    }

    /// returns data offsets array
    pub fn read_ply_header(reader: &mut impl BufRead) -> Result<(Header, Vec<usize>)> {
        let parser = parser::Parser::<ply::DefaultElement>::new();
        let header = parser.read_header(reader)?;
        if header.encoding != Encoding::BinaryLittleEndian {
            return Err(anyhow!("err: need BinaryLittleEndian"));
        }
        let vertex = header
            .elements
            .get("vertex")
            .ok_or(anyhow!("err: cannot read splat count"));

        let mut prop_sizes: Vec<usize> = vec![0; PLY_PROPERTIES.len()];

        let _ = vertex?.properties.iter().try_for_each(|(name, prop)| {
            if let Some(i) = PLY_PROPERTIES.iter().position(|&x| x == name) {
                prop_sizes[i] = Self::type_size(&prop.data_type)
                    .ok_or(anyhow!("err: unknow datatype{name}"))?;
            } else {
                eprintln!("warn:unknow property name {name}");
            }

            Ok(())
        });

        Ok((header, prop_sizes))
    }

    pub fn type_size(prop: &PropertyType) -> Option<usize> {
        match prop {
            ply::PropertyType::Scalar(ply::ScalarType::Float) => Some(4),
            // ply::PropertyType::Scalar(ply::ScalarType::Double) => Some(8),
            _ => None,
        }
    }
}

const PLY_PROPERTIES: &[&str] = &[
    "x",
    "y",
    "z",
    "nx",
    "ny",
    "nz",
    "f_dc_0",
    "f_dc_1",
    "f_dc_2",
    "f_rest_0",
    "f_rest_1",
    "f_rest_2",
    "f_rest_3",
    "f_rest_4",
    "f_rest_5",
    "f_rest_6",
    "f_rest_7",
    "f_rest_8",
    "f_rest_9",
    "f_rest_10",
    "f_rest_11",
    "f_rest_12",
    "f_rest_13",
    "f_rest_14",
    "f_rest_15",
    "f_rest_16",
    "f_rest_17",
    "f_rest_18",
    "f_rest_19",
    "f_rest_20",
    "f_rest_21",
    "f_rest_22",
    "f_rest_23",
    "f_rest_24",
    "f_rest_25",
    "f_rest_26",
    "f_rest_27",
    "f_rest_28",
    "f_rest_29",
    "f_rest_30",
    "f_rest_31",
    "f_rest_32",
    "f_rest_33",
    "f_rest_34",
    "f_rest_35",
    "f_rest_36",
    "f_rest_37",
    "f_rest_38",
    "f_rest_39",
    "f_rest_40",
    "f_rest_41",
    "f_rest_42",
    "f_rest_43",
    "f_rest_44",
    "opacity",
    "scale_0",
    "scale_1",
    "scale_2",
    "rot_0",
    "rot_1",
    "rot_2",
    "rot_3",
];
