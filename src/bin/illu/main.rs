use clap::Parser;

use crate::util::parse_resolution;

mod example;
mod util;

#[derive(Parser)]
#[command(name = "illuminator")]
#[command(about = "A simple yet powerful rendering tool.")]
struct Args {
    #[arg(short, long, value_name = "EXAMPLE_NAME")]
    example: Option<String>,

    #[arg(short, long, value_name = "PATH")]
    path: Option<String>,

    #[arg(short, long, value_name = "RESOLUTION")]
    res: Option<String>,
}

fn main() {
    let args = Args::parse();

    match &args.example {
        Some(name) => match name.as_str() {
            //  --example bvh [--path "./target/bvh.png"]
            "bvh" => example::bvh_example(args.path.as_deref()),
            //  --example 3dgs --path "./target/point_cloud.ply" [--res "256x256"]
            "3dgs" => {
                let res = {
                    let def_res = (256, 256);
                    args.res
                        .map_or(def_res, |res| parse_resolution(&res).unwrap_or(def_res))
                };

                example::gaussian_splatting_example(args.path.as_deref(), res);
            }
            _ => {
                eprintln!("Unknown example: {name}");
                std::process::exit(1);
            }
        },
        None => {
            println!("illuminator - A ray tracing and graphics library");
            println!("Use --help for more information");
        }
    }
}
