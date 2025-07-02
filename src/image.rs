bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ImageUsages: u32 {
        const TRANSFER_SRC                     = 0x00000001;
        const TRANSFER_DST                     = 0x00000002;
        const SAMPLED                          = 0x00000004;
        const STORAGE                          = 0x00000008;
        const COLOR_ATTACHMENT                 = 0x00000010;
        const DEPTH_STENCIL_ATTACHMENT         = 0x00000020;
        const TRANSIENT_ATTACHMENT             = 0x00000040;
        const INPUT_ATTACHMENT                 = 0x00000080;
        const HOST_TRANSFER                    = 0x00400000;
        const VIDEO_DECODE_DST                 = 0x00000400;
        const VIDEO_DECODE_SRC                 = 0x00000800;
        const VIDEO_DECODE_DPB                 = 0x00001000;
        const FRAGMENT_DENSITY_MAP             = 0x00000200;
        const FRAGMENT_SHADING_RATE_ATTACHMENT = 0x00000100;
        const VIDEO_ENCODE_DST                 = 0x00002000;
        const VIDEO_ENCODE_SRC                 = 0x00004000;
        const VIDEO_ENCODE_DPB                 = 0x00008000;
        const ATTACHMENT_FEEDBACK_LOOP         = 0x00080000;
    }
}

include!(concat!(env!("OUT_DIR"), "/format.rs"));
