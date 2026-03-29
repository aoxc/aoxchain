use dioxus::prelude::*;

#[component]
pub fn DomainSections() -> Element {
    rsx! {
        section {
            id: "bridge",
            class: "ecosystem glass",
            h2 { "Bridge & Interop" }
            p { "Native bridge policy, relayer health, and settlement routes are managed from this panel." }
        }

        section {
            id: "governance",
            class: "ecosystem glass",
            h2 { "Governance" }
            p { "Proposal pipeline, voting telemetry, and treasury execution are integrated with AOXC governance services." }
        }

        section {
            id: "staking",
            class: "ecosystem glass",
            h2 { "Staking" }
            p { "Delegation states, validator risk scores, and reward windows are streamed in near real-time." }
        }

        section {
            id: "ecosystem",
            class: "ecosystem glass",
            h2 { "Ecosystem Overview" }
            p { "AOX Hub connects monitoring, staking, bridge operations, observability pipelines, and governance automation in one desktop frame." }
        }
    }
}
