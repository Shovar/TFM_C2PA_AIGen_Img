# Data Provenance for AI Generated images using C2PA Schema

Software to add, edit and read provenance information for AI Generated images using C2PA Schema. It provides information about the GenAI model used to generate the image and the prompt introduced. The image generation is made using Stable diffusion with the Candle rust library.

    Options:
        --create            true     Creates an image using the prompt and uncond_prompt and adds a c2pa manifest to it. 
                                       Note: It should be used with at least the --prompt and --final_image parameters.
        --edit              true     Edits an image using the prompt, uncond_prompt and a source image 
                                     and adds a c2pa manifest to it. 
                                       Note: It should be used with at least the --prompt, --img2img and --final_image parameters
        --add              <VALUE>   Adds a c2pa manifest to a media file, displays the contents afterwards
        --edit_manifest    <VALUE>   Adds a c2pa manifest to a media file, displays the contents afterwards
        --read             <VALUE>   Prints the c2pa manifest contents of a media file; fails if no manifest is present
    
    Parameters:
        --prompt           <VALUE>   Prompt to generate the image
        --uncond-prompt    <VALUE>   Uncond prompt to generate the image
        --sd-version       <VALUE>   Stable diffusion options: V1_5, (default) V2_1, Xl, Turbo
        --final-image      <VALUE>   Name of the image to generate. Example: "Test_img.jpg"
        --img2img          <VALUE>   Path to the source image to be edited.
        --cpu                        Run the image Generation using the CPU rather than GPU
        --img2img_strength <VALUE>   The strength, indicates how much to transform the initial image. The
                                     value must be between 0 and 1, a value of 1 discards the initial image
                                     information.
```console
~>>  cargo build
~>>  cargo run -- --create true --prompt "Generate an Image of a Samurai" --uncond-prompt "He is not old" --sd-version turbo --final-image prueba.jpg
```

![Samurai image generated](https://github.com/Shovar/TFM_C2PA_AIGen_Img/blob/main/prueba.jpg?raw=true)
```console
  "active_manifest": "urn:uuid:741eddb5-1af2-44c7-ad10-deed1b4b3960",
   "manifests": {
    "urn:uuid:741eddb5-1af2-44c7-ad10-deed1b4b3960": {
      "claim_generator": "Toni-c2pa-GenAI-code/0.1 c2pa-rs/0.31.3",
      "title": "AI Generated Image",
      "format": "image/jpeg",
      "instance_id": "xmp:iid:3b25a73c-7925-4c26-a27d-e4cd223aab7e",
      "ingredients": [],
      "assertions": [
        {
          "label": "c2pa.actions",
          "data": {
            "actions": [
              {
                "action": "c2pa.created",
                "digitalSourceType": "https://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture",
                "softwareAgent": "Toni-c2pa-GenAI-code/0.1",
                "when": "2024-05-05T18:50:53.770467100+00:00"
              }
            ]
          }
        },
        {
          "label": "org.contentauth.test.model",
          "data": {
            "model": "Stable Diffusion",
            "timestamp": 1714935053,
            "version": "Turbo"
          }
        },
        {
          "label": "org.contentauth.test.prompt",
          "data": {
            "negative_prompt": "He is not old",
            "prompt": "Generate an image of a samurai"
          }
        }
      ],
      "signature_info": {
        "issuer": "C2PA Test Signing Cert",
        "cert_serial_number": "720724073027128164015125666832722375746636448153"
      },
      "label": "urn:uuid:741eddb5-1af2-44c7-ad10-deed1b4b3960"
    }
  }
```
