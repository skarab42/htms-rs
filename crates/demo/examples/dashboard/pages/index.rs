use std::time::Duration;

use htms::Template;
use tokio::time::sleep;

#[derive(Template, Default)]
#[template = "examples/dashboard/pages/index.html"]
pub struct Dashboard {}

impl DashboardRender for Dashboard {
    async fn user_stats_task() -> String {
        // Fast load (800ms)
        sleep(Duration::from_millis(800)).await;

        r#"
        <div class="card complete">
            <div class="status ready">READY</div>
            <div class="metric">1,247</div>
            <h3>Active Users</h3>
            <p class="label">Last 24 hours</p>
            <p>+23% from yesterday 📈</p>
        </div>
        "#
        .to_string()
    }

    async fn sales_data_task() -> String {
        // Medium load (1.5s)
        sleep(Duration::from_millis(1500)).await;

        r#"
        <div class="card complete">
            <div class="status ready">READY</div>
            <div class="metric">$12.4K</div>
            <h3>Revenue</h3>
            <p class="label">This month</p>
            <p>+45% growth 🚀</p>
        </div>
        "#
        .to_string()
    }

    async fn analytics_task() -> String {
        // Slow load (2.5s) - the grand finale
        sleep(Duration::from_millis(2500)).await;

        r#"
        <div class="card complete">
            <div class="status ready">READY</div>
            <div class="metric">98.7%</div>
            <h3>Performance</h3>
            <p class="label">Core Web Vitals</p>
            <p>HTMS rocks! ⚡</p>
        </div>
        "#
        .to_string()
    }
}
