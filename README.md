# Data Provenance for AI Generated images using C2PA Schema

Software to generate an image an add provenance data to it following the C2PA Schema. It can also just add or read the C2PA Manifest of an image
The image generation is made using Stable diffusion with the Candle rust library.

    Options:
        --add <VALUE>    adds a c2pa manifest to a media file, displays the contents afterwards
        --read <VALUE>   prints the c2pa manifest contents of a media file; fails if no manifest is present
        --create         Create an image using the prompt and uncond_prompt and adds a c2pa manifest to it
        --prompt         Prompt to generate the image
        --uncond-prompt  Uncond prompt to generate the image
        --sd-version     Stable diffusion options
        --final-image    Path and name of the created image
    -h, --help           Print help
    -V, --version        Print version

 ~>>  cargo run -- --create --prompt "Generate an Image of a Samurai" --uncond-prompt "He is not old" --sd-version turbo --final-image prueba.jpg
 ![Samurai image generated](https://github.com/Shovar/TFM_C2PA_AIGen_Img/blob/main/prueba.jpg?raw=true)
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
}
