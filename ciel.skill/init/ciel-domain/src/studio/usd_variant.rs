//! Port of usd_variant_manager.py — OpenUSD VariantSets + payload composition
//! stage emitter (LOD + material-state variants).

use serde_json::json;
use std::fs;
use std::path::Path;

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;

pub fn generate_usd_variant_stage(
    asset_name: &str,
    lod_files: &[String],
    out_usda_path: &str,
) -> Result<serde_json::Value, String> {
    let mut lod_variants = String::new();
    for (lod_idx, f_path) in lod_files.iter().enumerate() {
        let lod_name = format!("LOD{}", lod_idx);
        let base = Path::new(f_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        lod_variants.push_str(&format!(
            r#"
        "{lod_name}" (
            references = @{base}@
        ) {{
            custom int lod:level = {lod_idx}
        }}"#,
            lod_name = lod_name,
            base = base,
            lod_idx = lod_idx
        ));
    }

    let usda_content = format!(
        r#"#usda 1.0
(
    defaultPrim = "{asset_name}"
    doc = "CIEL Autonomous 3D Studio - Production OpenUSD Variant Asset"
    metersPerUnit = 1.0
    upAxis = "Z"
)

def Xform "{asset_name}" (
    assetInfo = {{
        string name = "{asset_name}"
        string author = "CIEL Autonomous 3D Studio"
        string version = "2.0.0"
    }}
    payload = @./{asset_name}_payload.usda@
    variants = {{
        string lod_variant = "LOD0"
        string material_state = "Pristine_Clean"
    }}
    prepend variantSets = ["lod_variant", "material_state"]
)
{{
    variantSet "lod_variant" = {{
        {lod_variants}
    }}

    variantSet "material_state" = {{
        "Pristine_Clean" {{
            rel material:binding = </{asset_name}/Materials/M_Clean>
        }}
        "Combat_Damaged" {{
            rel material:binding = </{asset_name}/Materials/M_Damaged>
        }}
        "Winter_Snow" {{
            rel material:binding = </{asset_name}/Materials/M_Snow>
        }}
    }}

    def Scope "Materials"
    {{
        def Material "M_Clean"
        {{
            token outputs:surface.connect = </{asset_name}/Materials/M_Clean/PBR.outputs:surface>
            def Shader "PBR"
            {{
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = (0.8, 0.8, 0.8)
                float inputs:roughness = 0.3
                float inputs:metallic = 0.0
                token outputs:surface
            }}
        }}

        def Material "M_Damaged"
        {{
            token outputs:surface.connect = </{asset_name}/Materials/M_Damaged/PBR.outputs:surface>
            def Shader "PBR"
            {{
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = (0.35, 0.32, 0.30)
                float inputs:roughness = 0.85
                float inputs:metallic = 0.2
                token outputs:surface
            }}
        }}

        def Material "M_Snow"
        {{
            token outputs:surface.connect = </{asset_name}/Materials/M_Snow/PBR.outputs:surface>
            def Shader "PBR"
            {{
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = (0.95, 0.95, 0.98)
                float inputs:roughness = 0.6
                float inputs:metallic = 0.0
                token outputs:surface
            }}
        }}
    }}
}}
"#,
        asset_name = asset_name,
        lod_variants = lod_variants
    );

    fs::write(out_usda_path, usda_content).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "SUCCESS",
        "asset_name": asset_name,
        "usd_stage": out_usda_path,
        "lod_variants_count": lod_files.len(),
        "material_states": ["Pristine_Clean", "Combat_Damaged", "Winter_Snow"]
    }))
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio usd-variant",
        argv,
        &[
            ArgSpec::value("name", Some('n'), "name").def("HeroAsset"),
            ArgSpec::multi("lods", None, "lods").req(),
            ArgSpec::value("out", Some('o'), "out").def("./Asset_Variants.usda"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let name = args.get_or("name", "HeroAsset");
    let lods = args.multi("lods");
    let out = args.get_or("out", "./Asset_Variants.usda");

    match generate_usd_variant_stage(&name, &lods, &out) {
        Ok(result) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&result, 2));
            } else {
                println!("\n[OpenUSD Variant Manager] Assembled Multi-Variant Stage:");
                println!(" -> Output USDA: {}", out);
                println!(" -> LOD Variants: {}", result["lod_variants_count"]);
                println!(
                    " -> Material State Variants: {}\n",
                    result["material_states"]
                        .as_array()
                        .map(|a| a
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", "))
                        .unwrap_or_default()
                );
            }
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
