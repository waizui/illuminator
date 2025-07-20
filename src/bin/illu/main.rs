use clap::Parser;

mod example;

#[derive(Parser)]
#[command(name = "illuminator")]
#[command(about = "A simple yet powerful rendering tool.")]
struct Args {
    /// Run an example: --example bvh [--path "./target/bvh.png"]
    #[arg(short, long, value_name = "EXAMPLE_NAME")]
    example: Option<String>,

    #[arg(short, long, value_name = "PATH")]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    match &args.example {
        Some(name) => match name.as_str() {
            "bvh" => example::bvh_example(args.path.as_deref()),
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
