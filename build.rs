use ciborium::ser;
use copy_to_output::copy_to_output;
use dart_parser_generator::{grammar, parser_generator};
use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        // generate parser
        let rules = grammar::get_dart_grammar();
        let transition_map = parser_generator::generate_parser(&rules, grammar::START_SYMBOL);

        // write binary to file
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("parser.bin");
        let writer = std::fs::File::create(dest_path).unwrap();
        ser::into_writer(&transition_map, writer).unwrap();
    }

    copy_to_output("template", &env::var("PROFILE").unwrap())
        .expect("failed to copy template files");
    copy_to_output("script", &env::var("PROFILE").unwrap()).expect("failed to copy script files");
    Ok(())
}
