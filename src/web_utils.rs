#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlImageElement;
#[cfg(target_arch = "wasm32")]
use wgpu::ExternalImageSource;

cfg_if::cfg_if! {

 if #[cfg(target_arch = "wasm32")] {
        // 1. Helper to load the image async
        // We wrap the onload/onerror callbacks in a Promise
        fn load_image(url: &str) -> Promise {

            Promise::new(&mut |resolve, reject| {
            web_sys::console::log_1(&JsValue::from_str("Promise executor started"));
            let window = web_sys::window().expect("no global `window` exists");
            let document = window.document().expect("should have a document on window");
            let img = document
                .create_element("img")
                .unwrap()
                .dyn_into::<HtmlImageElement>()
                .unwrap();

            img.set_cross_origin(Some("anonymous")); // Crucial for canvas security


            let img_clone = img.clone();
            // 1. Create the closures first
            let onload = Closure::once_into_js(move || {
                web_sys::console::log_1(&JsValue::from_str("Image LOADED"));
                let _ = resolve.call1(&JsValue::NULL, &JsValue::from(img_clone));
            });

            let onerror = Closure::once_into_js(move |err_event: JsValue| {
                web_sys::console::error_2(&JsValue::from_str("Image LOAD FAILED Details: "), &err_event);

            // Optional: If you want to see if it's a specific DOM Exception type
            // web_sys::console::dir(&err_event);
                let _ = reject.call0(&JsValue::NULL);
            });

            // 2. Attach the listeners
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            img.set_onerror(Some(onerror.as_ref().unchecked_ref()));

            // 3. Trigger the load
            web_sys::console::log_1(&JsValue::from_str("Image src set"));
            img.set_src(url);


            })
        }

        pub async fn load_texture_from_image_web(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            url: &str
        ) -> Result<wgpu::Texture, JsValue> {

            // 1. Load Image (Same helper as above)

            let promise = load_image(url);

            match wasm_bindgen_futures::JsFuture::from(promise).await
            {
               Ok(result) => {
                    let img: HtmlImageElement = result.dyn_into()?;

                    let width = img.width();
                    let height = img.height();

                    // 2. Create the WGPU Texture
                    let texture_size = wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    };

                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Image Texture"),
                        size: texture_size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        // TEXTURE_BINDING is required to use it in shaders
                        // COPY_DST is required to copy data into it
                        // RENDER_ATTACHMENT is required for copy_external_image_to_texture on some backends
                        usage: wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        view_formats: &[],
                    });

                    // 3. The Magic: Copy directly from DOM element to WGPU Texture
                    let image_source = wgpu::CopyExternalImageSourceInfo {
                        source: ExternalImageSource::HTMLImageElement(img),
                        origin: wgpu::Origin2d::ZERO,
                        flip_y: true, // Flip if your UVs require it
                    };

                    let image_destination = wgpu::CopyExternalImageDestInfo {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                            color_space: wgpu::PredefinedColorSpace::Srgb,
                            premultiplied_alpha: false,
                        };
                    queue.copy_external_image_to_texture(
                        &image_source,
                        image_destination,
                        texture_size,
                    );

                    Ok(texture)
                }

               Err(e) => {
                log::error!("Promise rejected: {:?}", e);
                return Err(e);
                }
            }
        }
        
        fn object_url_from_bytes(data: &[u8]) -> Result<String, JsValue> {
            use web_sys::Blob;

            let array = js_sys::Uint8Array::from(data);
            let blob = Blob::new_with_u8_array_sequence(&array.into())?;
            let url = web_sys::Url::create_object_url_with_blob(&blob)?;
            Ok(url)
        }
    }
}
