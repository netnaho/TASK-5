use dioxus::prelude::*;

#[component]
pub fn StatusBadge(status: String) -> Element {
    let class = match status.as_str() {
        "published" | "confirmed" | "approved" | "delivered" | "completed" | "mitigated" => "badge badge-success",
        "draft" | "new" | "pending" => "badge badge-default",
        "pending_approval" | "pending_step1" | "pending_step2" | "processing" => "badge badge-warning",
        "rejected" | "cancelled" | "failed" | "dead_letter" | "critical" | "open" => "badge badge-danger",
        "approved_scheduled" | "acknowledged" | "escalated" => "badge badge-info",
        "unpublished" | "false_positive" => "badge badge-muted",
        "high" => "badge badge-danger",
        "medium" => "badge badge-warning",
        "low" | "info" => "badge badge-info",
        _ => "badge badge-default",
    };
    rsx! { span { class: "{class}", "{status}" } }
}
