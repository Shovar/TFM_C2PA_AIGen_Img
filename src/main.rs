use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
mod create_img;
use c2pa::{create_signer, Ingredient, Manifest, ManifestStore, SigningAlg};
use c2pa::assertions::{c2pa_action, Action, Actions, SchemaDotOrgPerson}; //, CreativeWork, Exif
use chrono::prelude::{DateTime, Utc};
use clap::{arg, Command, Parser};
use regex::Regex;
use serde::Serialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {

     /// The path to the media file to add a manifest to.
     #[arg(long, value_name = "FILE")]
     add: Option<String>,
 
     /// The path to the media file to edit with a manifest.
     #[arg(long, value_name = "FILE")]
     edit: Option<String>,
 
     /// The path to the media file to read the manifest from.
     #[arg(long, value_name = "FILE")]
     read: Option<String>,

     #[arg(long)]
     create:Option<bool>,

    /// The prompt to be used for image generation.
    #[arg(
        long,
        default_value = "A very realistic photo of a rusty robot walking on a sandy beach"
    )]
    prompt: String,

    #[arg(long, default_value = "")]
    uncond_prompt: String,

    /// Run on CPU rather than on GPU.
    #[arg(long)]
    cpu: Option<bool>,

    /// Enable tracing (generates a trace-timestamp.json file).
    #[arg(long)]
    tracing: Option<bool>,

    /// The height in pixels of the generated image.
    #[arg(long)]
    height: Option<usize>,

    /// The width in pixels of the generated image.
    #[arg(long)]
    width: Option<usize>,

    /// The UNet weight file, in .safetensors format.
    #[arg(long, value_name = "FILE")]
    unet_weights: Option<String>,

    /// The CLIP weight file, in .safetensors format.
    #[arg(long, value_name = "FILE")]
    clip_weights: Option<String>,

    /// The VAE weight file, in .safetensors format.
    #[arg(long, value_name = "FILE")]
    vae_weights: Option<String>,

    #[arg(long, value_name = "FILE")]
    /// The file specifying the tokenizer to used for tokenization.
    tokenizer: Option<String>,

    /// The size of the sliced attention or 0 for automatic slicing (disabled by default)
    #[arg(long)]
    sliced_attention_size: Option<usize>,

    /// The number of steps to run the diffusion for.
    #[arg(long)]
    n_steps: Option<usize>,

    /// The number of samples to generate iteratively.
    #[arg(long, default_value_t = 1)]
    num_samples: usize,

    /// The numbers of samples to generate simultaneously.
    #[arg[long, default_value_t = 1]]
    bsize: usize,

    /// The name of the final image to generate.
    #[arg(long, value_name = "FILE", default_value = "sd_final.jpg")]
    final_image: Option<String>,

    #[arg(long, value_enum, default_value = "v2-1")]
    sd_version: StableDiffusionVersion,

    /// Generate intermediary images at each step.
    #[arg(long, action)]
    intermediary_images: bool,

    #[arg(long, value_name = "FILE")]
    img2img: Option<String>,

    /// The strength, indicates how much to transform the initial image. The
    /// value must be between 0 and 1, a value of 1 discards the initial image
    /// information.
    #[arg(long, default_value_t = 0.8)]
    img2img_strength: f64,
}

#[derive(Serialize,Debug, Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
enum StableDiffusionVersion {
    V1_5,
    V2_1,
    Xl,
    Turbo,
}

#[derive(Serialize)]
/* Custom assertion */
struct ModelData {
    model: String,
    version: StableDiffusionVersion,
    timestamp: u64
}

#[derive(Serialize)]
struct PromptData {
    prompt: String,
    negative_prompt: String
}

impl PromptData {
    fn new(prompt: String, negative_prompt:String) -> PromptData {
        PromptData {
            prompt: prompt,
            negative_prompt: negative_prompt
        }
    }
}

impl ModelData {
    fn new(model: String, version: StableDiffusionVersion ) -> ModelData {
        ModelData {
            model: model,
            version: version,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs()
        }
    }
}

/**
 * Creates a new `Manifest` for an image file. Represents a set of
 * actions performed when creating a new media file, for example, after
 * a digital image is taken.
 */
fn 
create_new_manifest (src_path: &String, dest_path: &String, PromptData: &PromptData, ModelData: &ModelData) -> Result<(), c2pa::Error> {
    let now: DateTime<Utc> = SystemTime::now().into();

    // ISO 8601 date and time format
    let now_string = now.to_rfc3339();

    // Initialized new Manifest with claim generator user agent string
    let mut manifest = Manifest::new("Toni-c2pa-GenAI-code/0.1".to_owned());

    // A new `Action` reflecting the creation of the media file    
    let created = Actions::new()
        .add_action(
            Action::new(c2pa_action::CREATED)
                .set_source_type("https://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture".to_owned())
                .set_software_agent("Toni-c2pa-GenAI-code/0.1")
                .set_when(now_string.clone())
        );

    // A new `CreativeWork`, defined in schema.org https://schema.org/CreativeWork
    // This represents the media created by the user, whose details are added to the 
    // `CreativeWork` as the author.
    /*let creative_work = CreativeWork::new()
        .add_author(
            SchemaDotOrgPerson::new()
                .set_name("Toni Garcia")
                  .expect("set name")
                .set_identifier("ToniGA_C2PA")
                  .expect("set identifier")
        )?;*/
        
    // Example Exif data to be embedded into the `Manifest`    
  /*  let exif = Exif::from_json_str(
        r#"{
        "@context" : {
          "exif": "http://ns.adobe.com/exif/1.0/"
        },
        "exif:GPSLatitude": "48,15.7068N",
        "exif:GPSLongitude": "16,15.9996W",
        "exif:GPSTimeStamp": "2023-08-23T19:12:45Z"
        }"#,
    ).expect("exif");*/

    // Sets some basics of the manifest
    manifest.set_title("AI Generated Image");
    manifest.set_format("image/jpeg");

   // Adds assertions about the content to the manifest
   manifest.add_assertion(&created)?;
   // manifest.add_assertion(&creative_work)?;
   // manifest.add_assertion(&exif)?;

    // Add custom data until this label to the manifest
    manifest.add_labeled_assertion("org.contentauth.test.model", &ModelData)?;
    manifest.add_labeled_assertion("org.contentauth.test.prompt", &PromptData)?;

    let source = PathBuf::from(src_path);
    let dest = PathBuf::from(dest_path);

    // Create a ps256 signer using certs and key files
    let signcert_path = "./certs/ps256.pub";
    let pkey_path = "./certs/ps256.pem";
    let signer = create_signer::from_files(signcert_path, pkey_path, SigningAlg::Ps256, None);

    // Signs and embeds the manifest into the destination file
    manifest.embed(&source, &dest, &*signer.unwrap())?;

    Ok(())
}

fn 
edit_media_with_action (src_path: &String, dest_path: &String, action: &str) -> Result<(), c2pa::Error> {
    // Manifests cannot be edited. To modify the contents of the manifest store, pull in earlier versions of the content
    // and its manifest as an ingredient.
    let parent = Ingredient::from_file(src_path)?;

    let mut manifest = Manifest::new("Toni-c2pa-GenAI-code/0.1".to_owned());

    let now: DateTime<Utc> = SystemTime::now().into();
    let now_string = now.to_rfc3339();

    // also add an action that we opened the file
    let actions = Actions::new()
        .add_action(
            Action::new(c2pa_action::OPENED)
                .set_parameter("identifier", parent.instance_id().to_owned())
                .expect("set identifier")
                .set_reason("editing")
                .set_software_agent("Toni-c2pa-GenAI-code/0.1")
                .set_when(now_string.clone())
        )
        .add_action(
            Action::new(action)
                .set_parameter("identifier", parent.instance_id().to_owned())
                .expect("set identifier")
                .set_reason("editing")
                .set_source_type("https://cv.iptc.org/newscodes/digitalsourcetype/minorHumanEdits".to_owned())
                .set_software_agent("Toni-c2pa-GenAI-code/0.1")
                .set_when(now_string.clone())
        );

    manifest.set_parent(parent)?;
    manifest.add_assertion(&actions)?;

    // Create a ps256 signer using certs and key files
    let signcert_path = "./certs/ps256.pub";
    let pkey_path = "./certs/ps256.pem";
    let signer = create_signer::from_files(signcert_path, pkey_path, SigningAlg::Ps256, None);

    manifest.embed(&src_path, &dest_path, &*signer.unwrap())?;

    Ok(())
}

fn 
read_manifest (path: &String) -> Result<(), c2pa::Error> {

    let manifest_store = ManifestStore::from_file(path)?;

    match manifest_store.validation_status() {
        Some(statuses) if !statuses.is_empty() => {
            println!("Loading manifest resulted in validation errors:");
            for status in statuses {
                println!("Validation status code: {}", status.code());
            }

            panic!("data validation errors");
        },
        _ => ()
    }

    println!("manifest store: {}", manifest_store);

    // active manifest is the most recently added manifest in the store.
    let manifest = manifest_store.get_active().unwrap();
    println!("active manifest: {}", manifest);

    println!("all manifests:\n----------------------");
    for iter in manifest_store.manifests().iter() {
        println!("manifest {},{}", iter.0, iter.1);
    }

    Ok(())
}

fn 
main() {

    // By default, just run with --add test_file.jpg
    // This adds a manifest to an output file test_file_c2pa.jpg
    // Read the contents of a file with a c2pa manifest via --read filename_c2pa.jpg
    let args = Args::parse();
    let add_path: Option<String>;
    if args.create != None {
        //Call generate image function
        add_path = args.final_image;
        let args2 = create_img::Args::parse();
        create_img::run(args2);
    }else {
        add_path  = args.add;
    }
    let edit_path = args.edit;
    let read_path = args.read; 

    let PromptData = PromptData::new(args.prompt.to_string(), args.uncond_prompt.to_string());
    let ModelData = ModelData::new("Stable Diffusion".to_string(), args.sd_version);

    match (add_path, edit_path, read_path) {
        (Some(file_path), edit_path_opt, read_path_opt) => {
            let file_path_regex = Regex::new(r"(.+)\.([a-zA-Z]+)").unwrap();
            let captures = file_path_regex.captures(&file_path).unwrap();

            // filename prefix; output media files (with added manifests) to to a new file with a suffix added.
            // For exmaple, destination file would be "test_file_c2pa.jpg" given an input of "test_file.jpg"
            let mut file_with_manifest = captures.get(1).unwrap().as_str().to_owned();

            // suffix for output file
            file_with_manifest.push_str("_c2pa");

            // filename extension
            file_with_manifest.push_str(".");
            file_with_manifest.push_str(captures.get(2).unwrap().as_str());

            // create a new manifest for the media file
            match create_new_manifest(&file_path, &file_with_manifest, &PromptData, &ModelData) {
                Ok(_) => read_manifest(&file_with_manifest).expect("manifest should be printed to stdout"),
                Err(e) => panic!("error creating manifest: {}", e)
            }

            match edit_path_opt {
                Some(edit_path) => {
                    // edit the media file with a series of actions
                    match edit_media_with_action(&edit_path, &edit_path, c2pa_action::CROPPED) {
                        Ok(_) => read_manifest(&edit_path).expect("manifest should be printed to stdout"),
                        Err(e) => panic!("cropping edit failed with {}", e)
                    };

                    match edit_media_with_action(&edit_path, &edit_path, c2pa_action::FILTERED) {
                        Ok(_) => read_manifest(&edit_path).expect("manifest should be printed to stdout"),
                        Err(e) => panic!("filtering edit failed with {}", e)
                    };

                    match edit_media_with_action(&edit_path, &edit_path, c2pa_action::COLOR_ADJUSTMENTS) {
                        Ok(_) => read_manifest(&edit_path).expect("manifest should be printed to stdout"),
                        Err(e) => panic!("color adjustment edit failed with {}", e)
                    };
                },
                _ => ()
            }
         
            // read the manifest
            match read_path_opt {
                Some(read_path) => read_manifest(&read_path).expect("manifest should be printed to stdout"),
                _ => ()
            }
        }
        (None, Some(file_path), read_path_opt) => {
        
            // edit the media file with a series of actions
            match edit_media_with_action(&file_path, &file_path, c2pa_action::CROPPED) {
                Ok(_) => read_manifest(&file_path).expect("manifest should be printed to stdout"),
                Err(e) => panic!("cropping edit failed with {}", e)
            };

            match edit_media_with_action(&file_path, &file_path, c2pa_action::FILTERED) {
                Ok(_) => read_manifest(&file_path).expect("manifest should be printed to stdout"),
                Err(e) => panic!("filtering edit failed with {}", e)
            };

            match edit_media_with_action(&file_path, &file_path, c2pa_action::COLOR_ADJUSTMENTS) {
                Ok(_) => read_manifest(&file_path).expect("manifest should be printed to stdout"),
                Err(e) => panic!("color adjustment edit failed with {}", e)
            };
            
            // read the manifest
            match read_path_opt {
                Some(read_path) => read_manifest(&read_path).expect("manifest should be printed to stdout"),
                _ => ()
            }
        }
        (None, None, Some(file_path)) => {
            read_manifest(&file_path).expect("manifest should be printed to stdout; perhaps no c2pa manifest is present?");
        }
        (None, None, None) => {
            println!("provide a path to a media file via --add <path> or --edit <path> or --read <path>");
        }
    }
}
