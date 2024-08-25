use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
mod create_img;
use c2pa::{create_signer, Ingredient, Manifest, ManifestStore, SigningAlg};
use c2pa::assertions::{c2pa_action, Action, Actions, SchemaDotOrgPerson , CreativeWork}; 
use chrono::prelude::{DateTime, Utc};
use clap::{arg, Parser}; // Command,
use regex::Regex;
use serde::Serialize;
use sysinfo::{System};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable creation of a new media file and the addition of its manifest.
    #[arg(long, conflicts_with = "edit", conflicts_with = "read", conflicts_with = "add", 
          conflicts_with = "edit_manifest", help = "Enable creation of a new media file and the addition of its manifest.")]
    create: Option<bool>,

    /// Enable editing of the media file and the addition of its manifest.
    #[arg(long, conflicts_with = "create", conflicts_with = "read", conflicts_with = "add", 
          conflicts_with = "edit_manifest", help = "Enable editing of the media file and the addition of its manifest.")]
    edit: Option<bool>,
 
    /// The path to the media file to read the manifest from.
    #[arg(long, value_name = "FILE", conflicts_with = "create", conflicts_with = "edit", conflicts_with = "add", 
          conflicts_with = "edit_manifest", help = "The path to the media file to read the manifest from.")]
    read: Option<String>,

    /// The path to the media file to add a manifest to.
    #[arg(long, value_name = "FILE", conflicts_with = "create", conflicts_with = "edit", conflicts_with = "read", 
           conflicts_with = "edit_manifest", help = "The path to the media file to add a manifest to.")]
    add: Option<String>,

    /// The path to the media file to add a manifest to.
    #[arg(long, value_name = "FILE", conflicts_with = "create", conflicts_with = "edit", conflicts_with = "read", 
        conflicts_with = "add", help = "The path to the media file to add a manifest to.")]
    edit_manifest: Option<String>,

    /// The prompt to be used for image generation.
    #[arg( long, default_value = "Generate an image of a futuristic city at night.", 
           help = "The prompt to be used for image generation.")]
    prompt: String,

    #[arg(long, default_value = "", help = "The negative prompt to be used for image generation.")]
    uncond_prompt: String,

    #[arg(long, default_value = "", help = "Author of the Image.")]
    author: String,

    #[arg(long, default_value = "Stable Diffusion", help = "AI model used to generate the image.")]
    model: String,

    /// Run on CPU rather than on GPU.
    #[arg(long)]
    cpu: Option<bool>,

    /// The name of the final image to generate.
    #[arg(long, value_name = "FILE", default_value = "sd_final.jpg", help = "The name of the final image to generate.")]
    final_image: Option<String>,

    #[arg(long, value_enum, default_value = "v2-1")]
    sd_version: StableDiffusionVersion,

    #[arg(long, value_name = "FILE", help = "The path of the source image to edit.")]
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
    None,
}

#[derive(Serialize)]
/* Custom assertion */
struct ModelData {
    model: String,
    version: StableDiffusionVersion,
    system_time: String
}

#[derive(Serialize)]
struct PromptData {
    prompt: String,
    negative_prompt: String,
    author: String
}

impl PromptData {
    fn new(prompt: String, negative_prompt:String, author:String) -> PromptData {
        PromptData {
            prompt: prompt,
            negative_prompt: negative_prompt,
            author: author
        }
    }
}

impl ModelData {
    fn new(model: String, version: StableDiffusionVersion ) -> ModelData {
        ModelData {
            model: model,
            version: version,
            system_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
        }
    }
}

/**
 * Creates a new `Manifest` for an image file. Represents a set of
 * actions performed when creating a new media file
 */
fn 
create_new_manifest (src_path: &String, dest_path: &String, prompt_data: &PromptData, model_data: &ModelData) -> Result<(), c2pa::Error> {
    let now: DateTime<Utc> = SystemTime::now().into();

    // ISO 8601 date and time format
    let now_string = now.to_rfc3339();

    // Initialized new Manifest with claim generator user agent string
    let mut manifest = Manifest::new("C2PA-PROV-GenAI/0.1".to_owned());
    let model_name = model_data.model.clone();
    // A new `Action` reflecting the creation of the media file    
    let created = Actions::new()
        .add_action(
            Action::new(c2pa_action::CREATED)
                .set_source_type("https://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia".to_owned())
                .set_software_agent(model_name.as_str())
                .set_when(now_string.clone())
        );

    // A new `CreativeWork`, defined in schema.org https://schema.org/CreativeWork
    // This represents the media created by the user, whose details are added to the 
    // `CreativeWork` as the author.
    let creative_work = CreativeWork::new()
        .add_author(
            SchemaDotOrgPerson::new()
                .set_name(prompt_data.author.to_string())
                  .expect("set name")
                .set_identifier("C2PA_GENAI_USER".to_string())
                  .expect("set identifier")
        )?;
        
    // Sets some basics of the manifest
    manifest.set_title("AI Generated Image");
    manifest.set_format("image/jpeg");

   // Adds assertions about the content to the manifest
   manifest.add_assertion(&created)?;
   manifest.add_assertion(&creative_work)?;

   // Add custom data until this label to the manifest
   manifest.add_labeled_assertion("edu.upc.fib.C2PA_Prov.model_data", &model_data)?;
   manifest.add_labeled_assertion("edu.upc.fib.C2PA_Prov.prompt_data", &prompt_data)?;

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
edit_manifest (src_path: &String, dest_path: &String, action: &str, prompt_data: &PromptData, model_data: &ModelData) -> Result<(), c2pa::Error> {
    // Manifests cannot be edited. To modify the contents of the manifest store, pull in earlier versions of the content
    // and its manifest as an ingredient.
    let parent = Ingredient::from_file(src_path)?;

    let mut manifest = Manifest::new("C2PA-PROV-GenAI/0.1".to_owned());

    let now: DateTime<Utc> = SystemTime::now().into();
    let now_string = now.to_rfc3339();

    let model_name = model_data.model.clone();

    // also add an action that we opened the file
    let actions = Actions::new()
        .add_action(
            Action::new(c2pa_action::OPENED)
                .set_parameter("identifier", parent.instance_id().to_owned())
                .expect("set identifier")
                .set_reason("editing")
                .set_software_agent(model_name.as_str())
                .set_when(now_string.clone())
        )
        .add_action(
            Action::new(action)
                .set_parameter("identifier", parent.instance_id().to_owned())
                .expect("set identifier")
                .set_reason("editing from Media file")
                .set_source_type("http://cv.iptc.org/newscodes/digitalsourcetype/compositeWithTrainedAlgorithmicMedia".to_owned())
                .set_software_agent(model_name.as_str())
                .set_when(now_string.clone())
        );

    manifest.set_parent(parent)?;
    manifest.add_assertion(&actions)?;
    // Add custom data until this label to the manifest
    manifest.add_labeled_assertion("edu.upc.fib.C2PA_Prov.model_data", &model_data)?;
    manifest.add_labeled_assertion("edu.upc.fib.C2PA_Prov.prompt_data", &prompt_data)?;

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

    //println!("manifest store: {}", manifest_store);

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
    let args = Args::parse();
    let add_path: Option<String>;
    let edit_path: Option<String>; 
    let source_path: String;
    let read_path = args.read;
    // if --create is set, generate an image with the prompt and save it to the final_image path
    if args.create != None {
        //Call generate image function
        add_path = args.final_image.clone();
        let args_create = create_img::Args::parse();
        let _ = create_img::run(args_create);
    }else {
        add_path  = args.add;
    }

    //if --edit is set, edit the image with the img2img path and save it to the final_image path
    if args.edit != None {
        edit_path     = args.final_image.clone();
        source_path   = args.img2img.expect("USAGE: source path is required. insert --img2img <path>").to_string();
        let args_edit = create_img::Args::parse();
        let _         = create_img::run(args_edit);
    }else {
        
        if args.edit_manifest != None {
            source_path = args.edit_manifest.expect("USAGE: source path is required. Insert --edit_manifest <path>").to_string();
            edit_path   = args.final_image.clone();
        }else {
            source_path = "".to_string();
            edit_path   = None;
        }
    }
    let mut prompt_data = PromptData::new(args.prompt.to_string(), args.uncond_prompt.to_string(), args.author.to_string());
    let mut model_data = ModelData::new(args.model.to_string(), args.sd_version);
    
    //Confirm that the model is supported
    if model_data.model != "Stable Diffusion" && args.create != None  {
        panic!("Model not supported. Use Stable Diffusion for image generation.");}
    else if model_data.model != "Stable Diffusion" && args.edit != None {
        panic!("Model not supported. Use Stable Diffusion for image editing.");}
    else if model_data.model != "Stable Diffusion"  {
        model_data.version = StableDiffusionVersion::None;
    }
    // If no author is informed, use the system name
    if prompt_data.author.to_string() == "" {
        prompt_data.author = System::host_name().expect("set name");
    }

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
            match create_new_manifest(&file_path, &file_with_manifest, &prompt_data, &model_data) {
                Ok(_) => read_manifest(&file_with_manifest).expect("manifest should be printed to stdout"),
                Err(e) => panic!("Error creating manifest: {}", e)
            }

            match edit_path_opt {
                Some(edit_path) => {
                    
                    // edit the media file with the source image as ingredient
                    match edit_manifest(&source_path, &edit_path, c2pa_action::EDITED, &prompt_data, &model_data) {
                        Ok(_) => read_manifest(&edit_path).expect("manifest should be printed to stdout"),
                        Err(e) => panic!("Error creating edit manifest: {}", e)
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
            match edit_manifest(&source_path, &file_path, c2pa_action::EDITED, &prompt_data, &model_data) {
                Ok(_) => read_manifest(&file_path).expect("manifest should be printed to stdout"),
                Err(e) => panic!("Error creating edit manifest: {}", e)
            };            

            match read_path_opt {
                Some(read_path) => read_manifest(&read_path).expect("manifest should be printed to stdout"),
                _ => ()
            }
        }
        (None, None, Some(file_path)) => {
            println!("Printing manifest");
            read_manifest(&file_path).expect("manifest should be printed to stdout; perhaps no c2pa manifest is present?");
        }
        (None, None, None) => {
            println!("USAGE: provide a path to a media file via -- --create --final_image <path> or -- --edit --img2img<path> --final_image<path> or -- --read <path>");
        }
    }
}
