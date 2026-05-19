#[cfg(test)]
#[path = "mainShould.rs"]
mod main_should;

#[cfg(test)]
#[path = "app/analysisShould.rs"]
mod app_analysis_should;
#[cfg(test)]
#[path = "app/firewall_serviceShould.rs"]
mod app_firewall_service_should;
#[cfg(test)]
#[path = "app/groupingShould.rs"]
mod app_grouping_should;
#[cfg(test)]
#[path = "app/inputShould.rs"]
mod app_input_should;
#[cfg(test)]
#[path = "app/installationShould.rs"]
mod app_installation_should;
#[cfg(test)]
#[path = "app/investigation_serviceShould.rs"]
mod app_investigation_service_should;
#[cfg(test)]
#[path = "app/ioShould.rs"]
mod app_io_should;
#[cfg(test)]
#[path = "app/modShould.rs"]
mod app_mod_should;
#[cfg(test)]
#[path = "app/nerdfontShould.rs"]
mod app_nerdfont_should;
#[cfg(test)]
#[path = "app/riskShould.rs"]
mod app_risk_should;
#[cfg(test)]
#[path = "app/typesShould.rs"]
mod app_types_should;

#[cfg(test)]
#[path = "app/network/modShould.rs"]
mod app_network_mod_should;
#[cfg(test)]
#[path = "app/process/modShould.rs"]
mod app_process_mod_should;

#[cfg(test)]
#[path = "app/ui/center_panelShould.rs"]
mod app_ui_center_panel_should;
#[cfg(test)]
#[path = "app/ui/dialogsShould.rs"]
mod app_ui_dialogs_should;
#[cfg(test)]
#[path = "app/ui/firewallShould.rs"]
mod app_ui_firewall_should;
#[cfg(test)]
#[path = "app/ui/footerShould.rs"]
mod app_ui_footer_should;
#[cfg(test)]
#[path = "app/ui/headerShould.rs"]
mod app_ui_header_should;
#[cfg(test)]
#[path = "app/ui/modShould.rs"]
mod app_ui_mod_should;
#[cfg(test)]
#[path = "app/ui/nav_sidebarShould.rs"]
mod app_ui_nav_sidebar_should;
#[cfg(test)]
#[path = "app/ui/sidebar_leftShould.rs"]
mod app_ui_sidebar_left_should;
#[cfg(test)]
#[path = "app/ui/sidebar_rightShould.rs"]
mod app_ui_sidebar_right_should;
#[cfg(test)]
#[path = "app/ui/themeShould.rs"]
mod app_ui_theme_should;
#[cfg(test)]
#[path = "app/ui/widgetsShould.rs"]
mod app_ui_widgets_should;

#[cfg(test)]
#[path = "config/modShould.rs"]
mod config_mod_should;
#[cfg(test)]
#[path = "i18n/modShould.rs"]
mod i18n_mod_should;
#[cfg(test)]
#[path = "i18n/translatorShould.rs"]
mod i18n_translator_should;
#[cfg(test)]
#[path = "resources/modShould.rs"]
mod resources_mod_should;
#[cfg(test)]
#[path = "services/api_clientShould.rs"]
mod services_api_client_should;
#[cfg(test)]
#[path = "services/geoip_serviceShould.rs"]
mod services_geoip_service_should;
#[cfg(test)]
#[path = "services/modShould.rs"]
mod services_mod_should;
#[cfg(test)]
#[path = "utils/api_builderShould.rs"]
mod utils_api_builder_should;
#[cfg(test)]
#[path = "utils/dbShould.rs"]
mod utils_db_should;
#[cfg(test)]
#[path = "utils/formattingShould.rs"]
mod utils_formatting_should;
#[cfg(test)]
#[path = "utils/icon_extractorShould.rs"]
mod utils_icon_extractor_should;
#[cfg(test)]
#[path = "utils/modShould.rs"]
mod utils_mod_should;
#[cfg(test)]
#[path = "utils/rate_limiterShould.rs"]
mod utils_rate_limiter_should;
#[cfg(test)]
#[path = "utils/signaturesShould.rs"]
mod utils_signatures_should;
#[cfg(test)]
#[path = "utils/whoisShould.rs"]
mod utils_whois_should;

#[cfg(test)]
#[path = "E2E/analysis_lifecycleShould.rs"]
mod e2e_analysis_lifecycle;
#[cfg(test)]
#[path = "E2E/export_and_investigationShould.rs"]
mod e2e_export_and_investigation;
#[cfg(test)]
#[path = "E2E/firewall_flowShould.rs"]
mod e2e_firewall_flow;
