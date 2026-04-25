#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityInputKind {
    Text,
    Image,
    Audio,
    Video,
    Document,
    WorkspaceResource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRouteKind {
    MainModel,
    NativeVision,
    RuntimeTool,
    AuxiliaryModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub input_kinds: &'static [CapabilityInputKind],
    pub preferred_routes: &'static [CapabilityRouteKind],
    pub recommended_tools: &'static [&'static str],
}

pub const CAPABILITIES: &[CapabilityDefinition] = &[
    CapabilityDefinition {
        id: "chat",
        label: "Chat",
        input_kinds: &[CapabilityInputKind::Text],
        preferred_routes: &[CapabilityRouteKind::MainModel],
        recommended_tools: &[],
    },
    CapabilityDefinition {
        id: "vision",
        label: "Vision",
        input_kinds: &[
            CapabilityInputKind::Image,
            CapabilityInputKind::WorkspaceResource,
        ],
        preferred_routes: &[
            CapabilityRouteKind::NativeVision,
            CapabilityRouteKind::RuntimeTool,
        ],
        recommended_tools: &["vision_analyze"],
    },
    CapabilityDefinition {
        id: "image_gen",
        label: "Image generation",
        input_kinds: &[CapabilityInputKind::Text],
        preferred_routes: &[CapabilityRouteKind::RuntimeTool],
        recommended_tools: &[],
    },
    CapabilityDefinition {
        id: "audio_stt",
        label: "Speech to text",
        input_kinds: &[CapabilityInputKind::Audio],
        preferred_routes: &[CapabilityRouteKind::AuxiliaryModel],
        recommended_tools: &[],
    },
    CapabilityDefinition {
        id: "audio_tts",
        label: "Text to speech",
        input_kinds: &[CapabilityInputKind::Text],
        preferred_routes: &[CapabilityRouteKind::RuntimeTool],
        recommended_tools: &[],
    },
];

pub fn capability_definition(id: &str) -> Option<&'static CapabilityDefinition> {
    CAPABILITIES.iter().find(|capability| capability.id == id)
}

pub fn recommended_tools_for_capability(id: &str) -> &'static [&'static str] {
    capability_definition(id)
        .map(|capability| capability.recommended_tools)
        .unwrap_or(&[])
}
