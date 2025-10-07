//! Monitoring UI components for TN5250R
//!
//! This module contains the monitoring dashboard and related UI functions.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::monitoring;

impl TN5250RApp {
    /// Display the comprehensive monitoring dashboard
    pub fn show_monitoring_dashboard_ui(&mut self, ui: &mut egui::Ui) {
        // Get current system health
        let monitoring = monitoring::MonitoringSystem::global();
        let report = monitoring.generate_report();

        ui.collapsing("System Health Overview", |ui| {
            // Overall system status
            let status_color = match report.system_health.status {
                monitoring::HealthStatus::Healthy => egui::Color32::GREEN,
                monitoring::HealthStatus::Warning => egui::Color32::YELLOW,
                monitoring::HealthStatus::Critical => egui::Color32::RED,
                monitoring::HealthStatus::Down => egui::Color32::DARK_RED,
            };

            ui.horizontal(|ui| {
                ui.label("System Status:");
                ui.colored_label(status_color, format!("{:?}", report.system_health.status));
                ui.label(format!("(Uptime: {:?})", report.system_health.uptime));
            });

            ui.label(format!("Last Check: {:?}", report.system_health.last_check.elapsed()));

            if report.system_health.consecutive_failures > 0 {
                ui.colored_label(egui::Color32::RED,
                    format!("Consecutive Failures: {}", report.system_health.consecutive_failures));
            }
        });

        ui.separator();

        // Component health status
        ui.collapsing("Component Health", |ui| {
            for (component_name, health) in &report.system_health.component_health {
                let status_color = match health.status {
                    monitoring::HealthStatus::Healthy => egui::Color32::GREEN,
                    monitoring::HealthStatus::Warning => egui::Color32::YELLOW,
                    monitoring::HealthStatus::Critical => egui::Color32::RED,
                    monitoring::HealthStatus::Down => egui::Color32::DARK_RED,
                };

                ui.horizontal(|ui| {
                    ui.colored_label(status_color, format!("{}: {:?}", component_name, health.status));
                    if health.error_count > 0 {
                        ui.colored_label(egui::Color32::RED, format!("({} errors)", health.error_count));
                    }
                    if health.response_time_ms > 0 {
                        ui.label(format!("({}ms avg)", health.response_time_ms));
                    }
                });
            }
        });

        ui.separator();

        // Performance metrics summary
        ui.collapsing("Performance Summary", |ui| {
            ui.label("Runtime Validation:");
            ui.horizontal(|ui| {
                ui.label("  Success Rate:");
                let total_validations = report.runtime_metrics.total_validations.load(Ordering::Relaxed);
                let successful_validations = report.runtime_metrics.successful_validations.load(Ordering::Relaxed);
                let success_rate = if total_validations > 0 {
                    (successful_validations as f64 / total_validations as f64) * 100.0
                } else {
                    100.0
                };
                let color = if success_rate > 90.0 { egui::Color32::GREEN } else { egui::Color32::YELLOW };
                ui.colored_label(color, format!("{success_rate:.1}%"));
            });

            ui.label("Quality Metrics:");
            ui.horizontal(|ui| {
                ui.label("  Code Quality:");
                let code_quality = report.quality_metrics.code_quality_score.load(Ordering::Relaxed);
                let quality_color = if code_quality > 80 { egui::Color32::GREEN } else { egui::Color32::YELLOW };
                ui.colored_label(quality_color, format!("{code_quality}/100"));
            });

            ui.horizontal(|ui| {
                ui.label("  Test Coverage:");
                let test_coverage = report.quality_metrics.test_coverage_percent.load(Ordering::Relaxed);
                let coverage_color = if test_coverage > 80 { egui::Color32::GREEN } else { egui::Color32::YELLOW };
                ui.colored_label(coverage_color, format!("{test_coverage}%"));
            });
        });

        ui.separator();

        // Security status
        ui.collapsing("Security Status", |ui| {
            ui.horizontal(|ui| {
                ui.label("Security Events:");
                let security_events = report.security_metrics.total_security_events.load(Ordering::Relaxed);
                ui.label(format!("{security_events}"));
            });

            ui.horizontal(|ui| {
                ui.label("Threats Mitigated:");
                let threats_mitigated = report.security_metrics.threat_mitigations.load(Ordering::Relaxed);
                ui.label(format!("{threats_mitigated}"));
            });

            let auth_failures = report.security_metrics.authentication_failures.load(Ordering::Relaxed);
            if auth_failures > 0 {
                ui.colored_label(egui::Color32::RED, format!("Auth Failures: {auth_failures}"));
            }
        });

        ui.separator();

        // Active alerts
        ui.collapsing("Active Alerts", |ui| {
            let active_alerts = report.recent_alerts.iter()
                .filter(|alert| !alert.resolved)
                .take(10)
                .collect::<Vec<_>>();

            if active_alerts.is_empty() {
                ui.colored_label(egui::Color32::GREEN, "No active alerts");
            } else {
                for alert in active_alerts {
                    let alert_color = match alert.level {
                        monitoring::AlertLevel::Info => egui::Color32::BLUE,
                        monitoring::AlertLevel::Warning => egui::Color32::YELLOW,
                        monitoring::AlertLevel::Critical => egui::Color32::RED,
                        monitoring::AlertLevel::Emergency => egui::Color32::DARK_RED,
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(alert_color, format!("{:?}", alert.level));
                        ui.label(format!("{}: {}", alert.component, alert.message));
                        if alert.acknowledged {
                            ui.colored_label(egui::Color32::GREEN, "ACK");
                        }
                    });
                }
            }
        });

        ui.separator();

        // Monitoring actions
        ui.horizontal(|ui| {
            if ui.button("Refresh Reports").clicked() {
                self.refresh_monitoring_reports();
            }

            if ui.button("Run Health Check").clicked() {
                let monitoring = monitoring::MonitoringSystem::global();
                match monitoring.perform_health_check() {
                    Ok(result) => {
                        eprintln!("MONITORING: Manual health check completed: {:?}", result.overall_status);
                    }
                    Err(e) => {
                        eprintln!("MONITORING: Manual health check failed: {e}");
                    }
                }
            }

            if ui.button("Generate Full Report").clicked() {
                self.generate_full_monitoring_report();
            }
        });

        // Display cached reports if available
        if !self.monitoring_reports.is_empty() {
            ui.separator();
            ui.collapsing("Detailed Reports", |ui| {
                for (report_name, report_content) in &self.monitoring_reports {
                    ui.collapsing(report_name, |ui| {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            ui.label(report_content);
                        });
                    });
                }
            });
        }
    }

    /// Refresh all monitoring reports
    pub fn refresh_monitoring_reports(&mut self) {
        let monitoring = monitoring::MonitoringSystem::global();

        // Generate and cache various reports
        let mut reports = HashMap::new();

        reports.insert("System Overview".to_string(), format!("{:?}", monitoring.generate_report().system_health.status));
        reports.insert("Performance Report".to_string(), monitoring.performance_monitor.generate_report());
        reports.insert("Security Report".to_string(), monitoring.security_monitor.generate_report());
        reports.insert("Integration Report".to_string(), monitoring.integration_monitor.generate_report());
        reports.insert("Quality Report".to_string(), monitoring.quality_assurance.generate_report());
        reports.insert("Alerting Report".to_string(), monitoring.alerting_system.generate_report());

        self.monitoring_reports = reports;
    }

    /// Generate a comprehensive monitoring report
    pub fn generate_full_monitoring_report(&mut self) {
        let monitoring = monitoring::MonitoringSystem::global();
        let report = monitoring.generate_report();

        let mut full_report = String::new();
        full_report.push_str("=== COMPREHENSIVE TN5250R MONITORING REPORT ===\n\n");

        // System health
        full_report.push_str(&format!("SYSTEM HEALTH: {:?}\n", report.system_health.status));
        full_report.push_str(&format!("UPTIME: {:?}\n", report.system_health.uptime));
        full_report.push_str(&format!("LAST CHECK: {:?} ago\n", report.system_health.last_check.elapsed()));

        if !report.system_health.component_health.is_empty() {
            full_report.push_str("\nCOMPONENT HEALTH:\n");
            for (name, health) in &report.system_health.component_health {
                full_report.push_str(&format!("  {}: {:?} ({} errors, {}ms avg)\n",
                    name, health.status, health.error_count, health.response_time_ms));
            }
        }

        // Performance summary
        full_report.push_str("\nPERFORMANCE SUMMARY:\n");
        let total_validations = report.runtime_metrics.total_validations.load(Ordering::Relaxed);
        let successful_validations = report.runtime_metrics.successful_validations.load(Ordering::Relaxed);
        let validation_success_rate = if total_validations > 0 {
            (successful_validations as f64 / total_validations as f64) * 100.0
        } else { 100.0 };
        full_report.push_str(&format!("  Validation Success Rate: {validation_success_rate:.1}%\n"));

        let code_quality = report.quality_metrics.code_quality_score.load(Ordering::Relaxed);
        let test_coverage = report.quality_metrics.test_coverage_percent.load(Ordering::Relaxed);
        full_report.push_str(&format!("  Code Quality Score: {code_quality}/100\n"));
        full_report.push_str(&format!("  Test Coverage: {test_coverage}%\n"));

        // Security summary
        full_report.push_str("\nSECURITY SUMMARY:\n");
        let total_security_events = report.security_metrics.total_security_events.load(Ordering::Relaxed);
        let threats_mitigated = report.security_metrics.threat_mitigations.load(Ordering::Relaxed);
        full_report.push_str(&format!("  Total Security Events: {total_security_events}\n"));
        full_report.push_str(&format!("  Threats Mitigated: {threats_mitigated}\n"));

        // Integration summary
        full_report.push_str("\nINTEGRATION SUMMARY:\n");
        let component_interactions = report.integration_metrics.component_interactions.load(Ordering::Relaxed);
        let integration_failures = report.integration_metrics.integration_failures.load(Ordering::Relaxed);
        full_report.push_str(&format!("  Component Interactions: {component_interactions}\n"));
        full_report.push_str(&format!("  Integration Failures: {integration_failures}\n"));

        // Recent alerts
        let active_alerts: Vec<_> = report.recent_alerts.iter().filter(|a| !a.resolved).collect();
        if !active_alerts.is_empty() {
            full_report.push_str("\nACTIVE ALERTS:\n");
            for alert in active_alerts.iter().take(5) {
                full_report.push_str(&format!("  [{}] {}: {}\n",
                    match alert.level {
                        monitoring::AlertLevel::Info => "INFO",
                        monitoring::AlertLevel::Warning => "WARN",
                        monitoring::AlertLevel::Critical => "CRIT",
                        monitoring::AlertLevel::Emergency => "EMERG",
                    },
                    alert.component, alert.message));
            }
        }

        // Cache the full report
        self.monitoring_reports.insert("Full System Report".to_string(), full_report);

        println!("MONITORING: Full system report generated and cached");
    }
}