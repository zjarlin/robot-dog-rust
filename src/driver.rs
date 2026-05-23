use crate::planner::{render_ros2_script, render_text, RobotPlan};

pub fn render_output(plan: &RobotPlan, ros2_script: bool) -> String {
    if ros2_script {
        render_ros2_script(plan)
    } else {
        render_text(plan)
    }
}
