use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    ptr,
};

use anyhow::{Ok, Result, anyhow};
use ply_rs::{
    parser::{self},
    ply::{self, Encoding, Header, PropertyType},
};

// continuous bytes gaussian splat
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawGaussian {
    pub pos: [f32; 3],
    pub nor: [f32; 3],
    pub dc0: [f32; 3],
    pub sh: [f32; 3 * 15],
    pub opacity: f32,
    pub scale: [f32; 3],
    pub rot: [f32; 4],
}

impl Default for RawGaussian {
    fn default() -> Self {
        RawGaussian {
            pos: [0.; 3],
            nor: [0.; 3],
            dc0: [0.; 3],
            sh: [0.; 3 * 15],
            opacity: 0.,
            scale: [0.; 3],
            rot: [0.; 4],
        }
    }
}

pub fn read_ply(path: &str) -> Result<Vec<RawGaussian>> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    // stride is in bytes
    let (header, prop_offset, stride) = read_ply_header(&mut reader)?;
    let splat_count = header.elements.get("vertex").unwrap().count;

    let mut buf: Vec<u8> = vec![0; stride * splat_count];
    reader.read_exact(&mut buf)?;

    let splats = vec![RawGaussian::default(); splat_count];

    unsafe {
        let base_prt = splats.as_ptr();
        (0..splat_count).for_each(|i| {
            let buf_base_offset = i * stride;
            let splat_ptr = base_prt.add(i) as *mut f32;

            // map every 4 bytes from buf to splat using prop_offset
            (0..PLY_PROPERTIES.len()).for_each(|j| {
                if prop_offset[j] != 0 || j == 0 {
                    // 0 is valid offset for first property
                    let buf_offset = buf_base_offset + prop_offset[j];
                    if buf_offset + 4 <= buf.len() {
                        let val = ptr::read_unaligned(buf.as_ptr().add(buf_offset) as *const f32);
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

    let mut stride = 0;

    for (i, (name, prop)) in v.properties.iter().enumerate() {
        let size = type_size(&prop.data_type)?;

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
        _ => {
            dbg!(prop);
            Err(anyhow!("err: unknow datatype"))
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
fn test_ply_read() -> Result<()> {
    let path = "./target/bicycle.ply";

    let splats0 = read_ply(path)?;
    let splats1 = read_ply(path)?;

    let same = splats0.iter().zip(splats1.iter()).all(|(s0, s1)| {
        s0.opacity == s1.opacity
            && s0.scale == s1.scale
            && s0.pos == s1.pos
            && s0.dc0 == s1.dc0
            && s0.sh == s1.sh
    });

    assert!(same);
    Ok(())
}
