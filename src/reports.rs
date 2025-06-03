use anyhow::Result;
use std::fs;
use std::path::Path;

use statement_cov::ProjectCoverage as StmtCoverage;
use cyclomatic_complexity::ProjectComplexity;
use branch_cov::ProjectBranchCoverage;

pub fn generate_html_report(
    stmt_coverage: Option<&StmtCoverage>,
    branch_coverage: Option<&ProjectBranchCoverage>, 
    complexity: Option<&ProjectComplexity>,
    output_path: &Path,
) -> Result<()> {
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SkanUJkod Analysis Report</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; text-align: center; margin-bottom: 30px; }}
        h2 {{ color: #34495e; border-bottom: 2px solid #3498db; padding-bottom: 10px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .metric {{ background: #ecf0f1; padding: 20px; border-radius: 6px; text-align: center; }}
        .metric h3 {{ margin: 0 0 10px 0; color: #2c3e50; }}
        .metric .value {{ font-size: 2em; font-weight: bold; color: #3498db; }}
        .coverage-high {{ color: #27ae60; }}
        .coverage-medium {{ color: #f39c12; }}
        .coverage-low {{ color: #e74c3c; }}
        .complexity-low {{ color: #27ae60; }}
        .complexity-medium {{ color: #f39c12; }}
        .complexity-high {{ color: #e74c3c; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #3498db; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .progress-bar {{ width: 100%; height: 20px; background-color: #ecf0f1; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #e74c3c 0%, #f39c12 50%, #27ae60 100%); transition: width 0.3s ease; }}
        .tab-container {{ margin-top: 30px; }}
        .tab-buttons {{ display: flex; border-bottom: 1px solid #ddd; }}
        .tab-button {{ padding: 10px 20px; background: none; border: none; cursor: pointer; font-size: 16px; }}
        .tab-button.active {{ background: #3498db; color: white; }}
        .tab-content {{ display: none; padding: 20px 0; }}
        .tab-content.active {{ display: block; }}
    </style>
    <script>
        function showTab(tabName) {{
            const tabs = document.querySelectorAll('.tab-content');
            const buttons = document.querySelectorAll('.tab-button');
            
            tabs.forEach(tab => tab.classList.remove('active'));
            buttons.forEach(btn => btn.classList.remove('active'));
            
            document.getElementById(tabName).classList.add('active');
            document.querySelector(`[onclick="showTab('${{tabName}}')"]`).classList.add('active');
        }}
    </script>
</head>
<body>
    <div class="container">
        <h1>🔍 SkanUJkod Analysis Report</h1>
        
        <div class="summary">
            {stmt_summary}
            {branch_summary}
            {complexity_summary}
        </div>

        <div class="tab-container">
            <div class="tab-buttons">
                {tab_buttons}
            </div>
            
            {tab_contents}
        </div>
    </div>
</body>
</html>"#,
        stmt_summary = generate_stmt_summary(stmt_coverage),
        branch_summary = generate_branch_summary(branch_coverage),
        complexity_summary = generate_complexity_summary(complexity),
        tab_buttons = generate_tab_buttons(stmt_coverage, branch_coverage, complexity),
        tab_contents = generate_tab_contents(stmt_coverage, branch_coverage, complexity),
    );

    fs::write(output_path, html_content)?;
    Ok(())
}

fn generate_stmt_summary(coverage: Option<&StmtCoverage>) -> String {
    match coverage {
        Some(cov) => {
            let class = if cov.overall_coverage >= 90.0 { "coverage-high" } 
                       else if cov.overall_coverage >= 70.0 { "coverage-medium" } 
                       else { "coverage-low" };
            format!(
                r#"<div class="metric">
                    <h3>Statement Coverage</h3>
                    <div class="value {}">{:.1}%</div>
                    <div>{}/{} statements covered</div>
                </div>"#,
                class, cov.overall_coverage, cov.covered_statements, cov.total_statements
            )
        },
        None => String::new(),
    }
}

fn generate_branch_summary(coverage: Option<&ProjectBranchCoverage>) -> String {
    match coverage {
        Some(cov) => {
            let class = if cov.overall_coverage_percentage >= 90.0 { "coverage-high" } 
                       else if cov.overall_coverage_percentage >= 70.0 { "coverage-medium" } 
                       else { "coverage-low" };
            format!(
                r#"<div class="metric">
                    <h3>Branch Coverage</h3>
                    <div class="value {}">{:.1}%</div>
                    <div>{}/{} branches covered</div>
                </div>"#,
                class, cov.overall_coverage_percentage, cov.covered_branches, cov.total_branches
            )
        },
        None => String::new(),
    }
}

fn generate_complexity_summary(complexity: Option<&ProjectComplexity>) -> String {
    match complexity {
        Some(comp) => {
            let class = if comp.average_complexity <= 5.0 { "complexity-low" } 
                       else if comp.average_complexity <= 10.0 { "complexity-medium" } 
                       else { "complexity-high" };
            format!(
                r#"<div class="metric">
                    <h3>Avg Complexity</h3>
                    <div class="value {}">{:.1}</div>
                    <div>Max: {} ({})</div>
                </div>"#,
                class, comp.average_complexity, comp.max_complexity, comp.max_complexity_function
            )
        },
        None => String::new(),
    }
}

fn generate_tab_buttons(
    stmt_coverage: Option<&StmtCoverage>,
    branch_coverage: Option<&ProjectBranchCoverage>,
    complexity: Option<&ProjectComplexity>,
) -> String {
    let mut buttons = Vec::new();
    
    if stmt_coverage.is_some() {
        buttons.push(r#"<button class="tab-button active" onclick="showTab('stmt-tab')">Statement Coverage</button>"#);
    }
    if branch_coverage.is_some() {
        buttons.push(r#"<button class="tab-button" onclick="showTab('branch-tab')">Branch Coverage</button>"#);
    }
    if complexity.is_some() {
        buttons.push(r#"<button class="tab-button" onclick="showTab('complexity-tab')">Complexity</button>"#);
    }
    
    buttons.join("\n")
}

fn generate_tab_contents(
    stmt_coverage: Option<&StmtCoverage>,
    branch_coverage: Option<&ProjectBranchCoverage>,
    complexity: Option<&ProjectComplexity>,
) -> String {
    let mut contents = Vec::new();
    
    if let Some(cov) = stmt_coverage {
        contents.push(generate_stmt_tab(cov));
    }
    if let Some(cov) = branch_coverage {
        contents.push(generate_branch_tab(cov));
    }
    if let Some(comp) = complexity {
        contents.push(generate_complexity_tab(comp));
    }
    
    contents.join("\n")
}

fn generate_stmt_tab(coverage: &StmtCoverage) -> String {
    let mut functions_html = String::new();
    let mut functions: Vec<_> = coverage.functions.iter().collect();
    functions.sort_by(|a, b| a.1.coverage_percentage.partial_cmp(&b.1.coverage_percentage).unwrap());

    for (func_name, func_coverage) in functions {
        let class = if func_coverage.coverage_percentage >= 90.0 { "coverage-high" } 
                   else if func_coverage.coverage_percentage >= 70.0 { "coverage-medium" } 
                   else { "coverage-low" };
        
        functions_html.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: {}%"></div>
                    </div>
                </td>
                <td class="{}">{:.1}%</td>
                <td>{}/{}</td>
            </tr>"#,
            func_name, func_coverage.coverage_percentage, class, 
            func_coverage.coverage_percentage,
            func_coverage.covered_statements, func_coverage.total_statements
        ));
    }

    format!(
        r#"<div id="stmt-tab" class="tab-content active">
            <h2>Statement Coverage Details</h2>
            <table>
                <thead>
                    <tr><th>Function</th><th>Coverage</th><th>Percentage</th><th>Covered/Total</th></tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        functions_html
    )
}

fn generate_branch_tab(coverage: &ProjectBranchCoverage) -> String {
    let mut functions_html = String::new();
    let mut functions: Vec<_> = coverage.functions.iter().collect();
    functions.sort_by(|a, b| a.1.coverage_percentage.partial_cmp(&b.1.coverage_percentage).unwrap());

    for (func_name, func_coverage) in functions {
        let class = if func_coverage.coverage_percentage >= 90.0 { "coverage-high" } 
                   else if func_coverage.coverage_percentage >= 70.0 { "coverage-medium" } 
                   else { "coverage-low" };
        
        functions_html.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: {}%"></div>
                    </div>
                </td>
                <td class="{}">{:.1}%</td>
                <td>{}/{}</td>
            </tr>"#,
            func_name, func_coverage.coverage_percentage, class,
            func_coverage.coverage_percentage,
            func_coverage.covered_branches, func_coverage.total_branches
        ));
    }

    format!(
        r#"<div id="branch-tab" class="tab-content">
            <h2>Branch Coverage Details</h2>
            <table>
                <thead>
                    <tr><th>Function</th><th>Coverage</th><th>Percentage</th><th>Covered/Total</th></tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        functions_html
    )
}

fn generate_complexity_tab(complexity: &ProjectComplexity) -> String {
    let mut functions_html = String::new();
    let mut functions: Vec<_> = complexity.functions.iter().collect();
    functions.sort_by(|a, b| b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity));

    for (name, func) in functions.iter().take(20) {
        let class = match func.cyclomatic_complexity {
            1..=5 => "complexity-low",
            6..=10 => "complexity-medium", 
            _ => "complexity-high",
        };
        
        functions_html.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            name, class, func.cyclomatic_complexity,
            func.cognitive_complexity, func.lines_of_code
        ));
    }

    format!(
        r#"<div id="complexity-tab" class="tab-content">
            <h2>Complexity Analysis (Top 20)</h2>
            <table>
                <thead>
                    <tr><th>Function</th><th>Cyclomatic</th><th>Cognitive</th><th>LOC</th></tr>
                </thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        functions_html
    )
}