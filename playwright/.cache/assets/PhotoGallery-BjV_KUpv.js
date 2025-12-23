/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { d as defineComponent, r as ref, i as reactive, j as onMounted, o as onUnmounted, a as createElementBlock, b as openBlock, F as Fragment, g as createCommentVNode, k as createBlock, l as renderList, f as createBaseVNode, m as normalizeClass, t as toDisplayString, p as createStaticVNode, q as Teleport, e as createVNode, T as Transition, w as withCtx, s as withModifiers } from './index-CkkE85_b.js';
import { _ as _export_sfc } from './_plugin-vue_export-helper-pcqpp-6-.js';

const _sfc_main =  defineComponent({
  __name: "PhotoGallery",
  props: {
    photos: {},
    loading: { type: Boolean }
  },
  setup(__props, { expose: __expose }) {
    __expose();
    const props = __props;
    const lightbox = ref(null);
    const imageLoaded = reactive({});
    function openLightbox(url) {
      lightbox.value = url;
    }
    function closeLightbox() {
      lightbox.value = null;
    }
    function onImageLoad(url) {
      imageLoaded[url] = true;
    }
    function nextImage() {
      if (!lightbox.value) return;
      const idx = props.photos.indexOf(lightbox.value);
      if (idx < props.photos.length - 1) lightbox.value = props.photos[idx + 1];
    }
    function prevImage() {
      if (!lightbox.value) return;
      const idx = props.photos.indexOf(lightbox.value);
      if (idx > 0) lightbox.value = props.photos[idx - 1];
    }
    function onKeydown(e) {
      if (!lightbox.value) return;
      if (e.key === "Escape") closeLightbox();
      if (e.key === "ArrowRight") nextImage();
      if (e.key === "ArrowLeft") prevImage();
    }
    onMounted(() => window.addEventListener("keydown", onKeydown));
    onUnmounted(() => window.removeEventListener("keydown", onKeydown));
    const __returned__ = { props, lightbox, imageLoaded, openLightbox, closeLightbox, onImageLoad, nextImage, prevImage, onKeydown };
    Object.defineProperty(__returned__, "__isScriptSetup", { enumerable: false, value: true });
    return __returned__;
  }
});

const _hoisted_1 = {
  key: 0,
  class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4"
};
const _hoisted_2 = { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4" };
const _hoisted_3 = ["onClick"];
const _hoisted_4 = ["src", "onLoad"];
const _hoisted_5 = { class: "absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-end p-4" };
const _hoisted_6 = { class: "translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-300 delay-75" };
const _hoisted_7 = { class: "text-white text-xs font-medium truncate w-full" };
const _hoisted_8 = ["src"];
const _hoisted_9 = { class: "absolute bottom-8 left-1/2 -translate-x-1/2 px-6 py-3 bg-white/10 backdrop-blur-md rounded-full border border-white/10 text-sm text-white/80" };
function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  return openBlock(), createElementBlock(
    Fragment,
    null,
    [
      createCommentVNode(" Loading Skeletons "),
      $props.loading ? (openBlock(), createElementBlock("div", _hoisted_1, [
        (openBlock(), createElementBlock(
          Fragment,
          null,
          renderList(10, (i) => {
            return createBaseVNode("div", {
              key: i,
              class: "aspect-square bg-white/5 rounded-xl animate-pulse"
            });
          }),
          64
          
        ))
      ])) : $props.photos.length ? (openBlock(), createElementBlock(
        Fragment,
        { key: 1 },
        [
          createCommentVNode(" Gallery Grid "),
          createBaseVNode("div", _hoisted_2, [
            (openBlock(true), createElementBlock(
              Fragment,
              null,
              renderList($props.photos, (url) => {
                return openBlock(), createElementBlock("div", {
                  key: url,
                  onClick: ($event) => $setup.openLightbox(url),
                  class: "group relative aspect-square rounded-xl overflow-hidden cursor-pointer bg-white/5"
                }, [
                  createCommentVNode(" Image "),
                  createBaseVNode("img", {
                    src: url,
                    onLoad: ($event) => $setup.onImageLoad(url),
                    class: normalizeClass(["w-full h-full object-cover transition-transform duration-700 ease-out group-hover:scale-110", { "opacity-0": !$setup.imageLoaded[url] }]),
                    loading: "lazy"
                  }, null, 42, _hoisted_4),
                  createCommentVNode(" Premium Overlay "),
                  createBaseVNode("div", _hoisted_5, [
                    createBaseVNode("div", _hoisted_6, [
                      createBaseVNode(
                        "p",
                        _hoisted_7,
                        toDisplayString(url.split("/").pop()),
                        1
                        
                      )
                    ])
                  ])
                ], 8, _hoisted_3);
              }),
              128
              
            ))
          ])
        ],
        2112
        
      )) : (openBlock(), createElementBlock(
        Fragment,
        { key: 2 },
        [
          createCommentVNode(" Empty State "),
          _cache[0] || (_cache[0] = createStaticVNode('<div class="h-96 flex flex-col items-center justify-center text-gray-500" data-v-51718b6f><div class="w-16 h-16 rounded-2xl bg-white/5 flex items-center justify-center mb-4" data-v-51718b6f><svg class="w-6 h-6 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor" data-v-51718b6f><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" data-v-51718b6f></path></svg></div><p class="text-sm" data-v-51718b6f>No photos found</p></div>', 1))
        ],
        2112
        
      )),
      createCommentVNode(" Cinematic Lightbox "),
      (openBlock(), createBlock(Teleport, { to: "body" }, [
        createVNode(Transition, { name: "lightbox" }, {
          default: withCtx(() => [
            $setup.lightbox ? (openBlock(), createElementBlock("div", {
              key: 0,
              class: "fixed inset-0 z-[100] bg-black/95 backdrop-blur-xl flex items-center justify-center",
              onClick: withModifiers($setup.closeLightbox, ["self"])
            }, [
              createCommentVNode(" Controls "),
              createBaseVNode("button", {
                onClick: $setup.closeLightbox,
                class: "absolute top-6 right-6 p-4 text-white/50 hover:text-white transition-colors"
              }, [..._cache[1] || (_cache[1] = [
                createBaseVNode(
                  "svg",
                  {
                    class: "w-6 h-6",
                    fill: "none",
                    viewBox: "0 0 24 24",
                    stroke: "currentColor"
                  },
                  [
                    createBaseVNode("path", {
                      "stroke-linecap": "round",
                      "stroke-linejoin": "round",
                      "stroke-width": "2",
                      d: "M6 18L18 6M6 6l12 12"
                    })
                  ],
                  -1
                  
                )
              ])]),
              $props.photos.indexOf($setup.lightbox) > 0 ? (openBlock(), createElementBlock("button", {
                key: 0,
                onClick: withModifiers($setup.prevImage, ["stop"]),
                class: "absolute left-6 p-4 text-white/50 hover:text-white hover:scale-110 transition-all"
              }, [..._cache[2] || (_cache[2] = [
                createBaseVNode(
                  "svg",
                  {
                    class: "w-8 h-8",
                    fill: "none",
                    viewBox: "0 0 24 24",
                    stroke: "currentColor"
                  },
                  [
                    createBaseVNode("path", {
                      "stroke-linecap": "round",
                      "stroke-linejoin": "round",
                      "stroke-width": "2",
                      d: "M15 19l-7-7 7-7"
                    })
                  ],
                  -1
                  
                )
              ])])) : createCommentVNode("v-if", true),
              $props.photos.indexOf($setup.lightbox) < $props.photos.length - 1 ? (openBlock(), createElementBlock("button", {
                key: 1,
                onClick: withModifiers($setup.nextImage, ["stop"]),
                class: "absolute right-6 p-4 text-white/50 hover:text-white hover:scale-110 transition-all"
              }, [..._cache[3] || (_cache[3] = [
                createBaseVNode(
                  "svg",
                  {
                    class: "w-8 h-8",
                    fill: "none",
                    viewBox: "0 0 24 24",
                    stroke: "currentColor"
                  },
                  [
                    createBaseVNode("path", {
                      "stroke-linecap": "round",
                      "stroke-linejoin": "round",
                      "stroke-width": "2",
                      d: "M9 5l7 7-7 7"
                    })
                  ],
                  -1
                  
                )
              ])])) : createCommentVNode("v-if", true),
              createCommentVNode(" Main Image "),
              createBaseVNode("img", {
                src: $setup.lightbox,
                class: "max-w-[95vw] max-h-[90vh] object-contain rounded-lg shadow-2xl animate-scale-up"
              }, null, 8, _hoisted_8),
              createCommentVNode(" Caption "),
              createBaseVNode(
                "div",
                _hoisted_9,
                toDisplayString($setup.lightbox.split("/").pop()),
                1
                
              )
            ])) : createCommentVNode("v-if", true)
          ]),
          _: 1
          
        })
      ]))
    ],
    64
    
  );
}
const PhotoGallery =  _export_sfc(_sfc_main, [["render", _sfc_render], ["__scopeId", "data-v-51718b6f"], ["__file", "PhotoGallery.vue"]]);

export { PhotoGallery as default };