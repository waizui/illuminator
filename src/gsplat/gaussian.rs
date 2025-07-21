use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Ok, Result, anyhow};
use ply_rs::{
    parser::{self},
    ply::{self, Encoding, Header, PropertyType},
};

use crate::{
    core::{quaternion::Quat, tensor::Vec3f},
    raycast::{Hit, Ray, Raycast},
};

#[derive(Debug, Clone, Copy)]
pub struct Gaussian {
    pub pos: Vec3f,
    pub nor: Vec3f,
    pub col: Vec3f,
    pub sh: [Vec3f; 15],
    pub opacity: f32,
    pub scale: Vec3f,
    pub rot: Quat,
}

impl Raycast for Gaussian {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        todo!()
    }
}

pub struct GaussianScene {}

impl GaussianScene {
    pub fn read_ply(&mut self, path: &str) -> Result<Vec<Gaussian>> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

        let (header, stride) = Self::read_ply_header(&mut reader)?;

        let splat_count = header.elements.get("vertex").unwrap().count;
        let splats: Vec<Gaussian> = (0..splat_count)
            .map(|_| {
                // TODO: parse data
                todo!()
            })
            .collect();

        Ok(splats)
    }

    pub fn read_ply_header(reader: &mut impl BufRead) -> Result<(Header, usize)> {
        let parser = parser::Parser::<ply::DefaultElement>::new();
        let header = parser.read_header(reader)?;
        if header.encoding != Encoding::BinaryLittleEndian {
            return Err(anyhow!("err: need BinaryLittleEndian"));
        }
        let vertex = header
            .elements
            .get("vertex")
            .ok_or(anyhow!("err: cannot read splat count"));

        let stride = vertex?
            .properties
            .iter()
            .zip(PLY_PROPERTIES.iter())
            .try_fold(0, |acc, ((a, prop), b)| {
                if a != *b {
                    eprintln!("warn:unknow property name {a}");
                }
                Self::type_size(&prop.data_type).map(|size| acc + size)
            });

        Ok((header, stride.ok_or(anyhow!("err:unknow data type"))?))
    }

    pub fn type_size(prop: &PropertyType) -> Option<usize> {
        match prop {
            ply::PropertyType::Scalar(ply::ScalarType::Float) => Some(4),
            ply::PropertyType::Scalar(ply::ScalarType::Double) => Some(8),
            ply::PropertyType::Scalar(ply::ScalarType::UChar) => Some(1),
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
