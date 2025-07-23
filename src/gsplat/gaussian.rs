use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    ptr, usize,
};

use anyhow::{Ok, Result, anyhow};
use bytemuck::{Pod, Zeroable};
use ply_rs::{
    parser::{self},
    ply::{self, Encoding, Header, PropertyType},
};
use rayon::prelude::*;

use crate::{
    core::quaternion::Quat,
    prelude::{BVH, Vec3f},
    raycast::{Hit, Ray, Raycast, bounds::Bounds3f, primitive::Primitive},
};

// continuous bytes gaussian splat
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
        const SH_C0: f32 = 0.2820948;
        let col = Vec3f::vec(input.dc0) * SH_C0 + 0.5;

        let sh = std::array::from_fn(|i| {
            let reodered_sh = [input.sh[i], input.sh[i + 15], input.sh[i + 30]];
            Vec3f::vec(reodered_sh)
        });

        Splat {
            pos: Vec3f::vec(input.pos),
            nor: Vec3f::vec(input.nor),
            col,
            sh,
            opacity: input.opacity,
            scale: Vec3f::vec(input.scale),
            rot: Quat::from_array(input.rot),
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
        let one = Vec3f::vec([0.01; 3]);
        Bounds3f {
            min: self.pos - one,
            max: self.pos + one,
        }
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// a collection of splats
pub struct GaussianScene {
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

        bvh.build(129, true);

        Ok(GaussianScene { bvh })
    }

    pub fn read_ply(path: &str) -> Result<Vec<InputSplat>> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);

        // stride is in bytes
        let (header, prop_offset, stride) = Self::read_ply_header(&mut reader)?;
        let splat_count = header.elements.get("vertex").unwrap().count;

        let mut buf: Vec<u8> = vec![0; stride * splat_count];
        reader.read_exact(&mut buf)?;

        let mut splats = vec![InputSplat::zeroed(); splat_count];

        unsafe {
            (0..splat_count).for_each(|i| {
                let base_offset = i * stride;
                let splat_ptr = &mut splats[i] as *mut InputSplat as *mut f32;

                // map every 4 bytes from buf to splat using prop_offset
                (0..PLY_PROPERTIES.len()).for_each(|j| {
                    if prop_offset[j] != 0 || j == 0 {
                        // 0 is valid offset for first property
                        let buf_offset = base_offset + prop_offset[j];
                        if buf_offset + 4 <= buf.len() {
                            let val =
                                ptr::read_unaligned(buf.as_ptr().add(buf_offset) as *const f32);
                            ptr::write(splat_ptr.add(j), val);
                        }
                    }
                });
            });
        }

        Ok(splats)
    }

    /// returns data offsets array
    pub fn read_ply_header(reader: &mut impl BufRead) -> Result<(Header, Vec<usize>, usize)> {
        let parser = parser::Parser::<ply::DefaultElement>::new();
        let header = parser.read_header(reader)?;
        if header.encoding != Encoding::BinaryLittleEndian {
            return Err(anyhow!("err: need BinaryLittleEndian"));
        }

        let vertex = header
            .elements
            .get("vertex")
            .ok_or(anyhow!("err: cannot read splat count"));

        let mut prop_offset: Vec<usize> = vec![0; PLY_PROPERTIES.len()];

        let v = vertex?;
        let size= std::mem::size_of::<InputSplat>();
        println!("inputsize: {size}");

        let mut stride = 0;

        for (i, (name, prop)) in v.properties.iter().enumerate() {
            let size = Self::type_size(&prop.data_type)?;

            if let Some(j) = PLY_PROPERTIES.iter().position(|&x| x == name) {
                prop_offset[j] = i * size;
            } else {
                eprintln!("warn:unknow property name {name}");
            }

            stride += size;
        }

        Ok((header, prop_offset, stride))
    }

    pub fn type_size(prop: &PropertyType) -> Result<usize> {
        match prop {
            ply::PropertyType::Scalar(ply::ScalarType::Float) => Ok(4),
            // ply::PropertyType::Scalar(ply::ScalarType::Double) => Some(8),
            _ => Err(anyhow!("err: unknow datatype")),
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

#[test]
fn test_ply_read() {
    use crate::img::*;
    use image::{Rgb, RgbImage};

    let path = "./target/obj_011.ply";
    let gs = GaussianScene::from_ply(path).unwrap();

    let (w, h) = (32, 32);
    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (lw, ly) = (8., 8.);
            let (x, y) = (
                iw as f32 * lw / (w - 1) as f32,
                (h - ih) as f32 * ly / (h - 1) as f32,
            );

            let org = Vec3f::vec([x - lw / 2., y - ly / 2., 1025.]);
            let dir = Vec3f::vec([0., 0., -1.]);
            let ray = Ray::new(org, dir);

            if let Some((_, i)) = gs.bvh.raycast_node(&ray) {
                let prim = &gs.bvh.primitives[i];
                let splat = prim.as_any().downcast_ref::<Splat>().unwrap();
                let col = splat.col * 255.;
                let r = col[0] as u8;
                let g = col[1] as u8;
                let b = col[2] as u8;
                *pix = Rgb([r, g, b]);
            }
        });

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save("./target/gs_example.png")
        .expect("Failed to save BVH example image");
}
