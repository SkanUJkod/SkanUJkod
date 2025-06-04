use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use branch_cov::ProjectBranchCoverage;
use cfg::{build_cfgs_for_file, parse_project, to_dot};
use cyclomatic_complexity::ProjectComplexity;
use statement_cov::ProjectCoverage as StmtCoverage;

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
            let class = if cov.overall_coverage >= 90.0 {
                "coverage-high"
            } else if cov.overall_coverage >= 70.0 {
                "coverage-medium"
            } else {
                "coverage-low"
            };
            format!(
                r#"<div class="metric">
                    <h3>Statement Coverage</h3>
                    <div class="value {}">{:.1}%</div>
                    <div>{}/{} statements covered</div>
                </div>"#,
                class, cov.overall_coverage, cov.covered_statements, cov.total_statements
            )
        }
        None => String::new(),
    }
}

fn generate_branch_summary(coverage: Option<&ProjectBranchCoverage>) -> String {
    match coverage {
        Some(cov) => {
            let class = if cov.overall_coverage_percentage >= 90.0 {
                "coverage-high"
            } else if cov.overall_coverage_percentage >= 70.0 {
                "coverage-medium"
            } else {
                "coverage-low"
            };
            format!(
                r#"<div class="metric">
                    <h3>Branch Coverage</h3>
                    <div class="value {}">{:.1}%</div>
                    <div>{}/{} branches covered</div>
                </div>"#,
                class, cov.overall_coverage_percentage, cov.covered_branches, cov.total_branches
            )
        }
        None => String::new(),
    }
}

fn generate_complexity_summary(complexity: Option<&ProjectComplexity>) -> String {
    match complexity {
        Some(comp) => {
            let class = if comp.average_complexity <= 5.0 {
                "complexity-low"
            } else if comp.average_complexity <= 10.0 {
                "complexity-medium"
            } else {
                "complexity-high"
            };
            format!(
                r#"<div class="metric">
                    <h3>Avg Complexity</h3>
                    <div class="value {}">{:.1}</div>
                    <div>Max: {} ({})</div>
                </div>"#,
                class, comp.average_complexity, comp.max_complexity, comp.max_complexity_function
            )
        }
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
        buttons.push(
            r#"<button class="tab-button" onclick="showTab('complexity-tab')">Complexity</button>"#,
        );
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
    functions.sort_by(|a, b| {
        a.1.coverage_percentage
            .partial_cmp(&b.1.coverage_percentage)
            .unwrap()
    });

    for (func_name, func_coverage) in functions {
        let class = if func_coverage.coverage_percentage >= 90.0 {
            "coverage-high"
        } else if func_coverage.coverage_percentage >= 70.0 {
            "coverage-medium"
        } else {
            "coverage-low"
        };

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
            func_name,
            func_coverage.coverage_percentage,
            class,
            func_coverage.coverage_percentage,
            func_coverage.covered_statements,
            func_coverage.total_statements
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
    functions.sort_by(|a, b| {
        a.1.coverage_percentage
            .partial_cmp(&b.1.coverage_percentage)
            .unwrap()
    });

    for (func_name, func_coverage) in functions {
        let class = if func_coverage.coverage_percentage >= 90.0 {
            "coverage-high"
        } else if func_coverage.coverage_percentage >= 70.0 {
            "coverage-medium"
        } else {
            "coverage-low"
        };

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
            func_name,
            func_coverage.coverage_percentage,
            class,
            func_coverage.coverage_percentage,
            func_coverage.covered_branches,
            func_coverage.total_branches
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
            name, class, func.cyclomatic_complexity, func.cognitive_complexity, func.lines_of_code
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

pub fn generate_cfg_html_gallery(
    project_path: &Path,
    output_path: &Path,
    generate_images: bool,
) -> Result<()> {
    // Parse project and build CFGs
    let (fset, objs, files) = parse_project(project_path)?;

    let mut cfgs_map = HashMap::new();
    for pf in &files {
        let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
        cfgs_map.extend(per_file_map);
    }

    // Create output directory structure
    let output_dir = output_path.parent().unwrap_or(Path::new("."));
    let graphs_dir = output_dir.join("graphs");
    fs::create_dir_all(&graphs_dir)?;

    let mut graph_entries = Vec::new();

    for (fname, graph) in &cfgs_map {
        let dot_content = to_dot(graph, fname);

        if generate_images {
            // Generate SVG image
            let svg_path = graphs_dir.join(format!("{}.svg", fname));
            crate::cli::generate_image_from_dot(&dot_content, &svg_path, crate::ImageFormat::Svg)?;

            // Add entry for HTML
            graph_entries.push(GraphEntry {
                function_name: fname.clone(),
                svg_path: format!("graphs/{}.svg", fname),
                nodes: graph.blocks.len(),
                edges: calculate_edge_count(graph),
                complexity: calculate_cfg_complexity(graph),
            });
        } else {
            // Just save DOT file
            let dot_path = graphs_dir.join(format!("{}.dot", fname));
            fs::write(&dot_path, &dot_content)?;

            graph_entries.push(GraphEntry {
                function_name: fname.clone(),
                svg_path: format!("graphs/{}.dot", fname),
                nodes: graph.blocks.len(),
                edges: calculate_edge_count(graph),
                complexity: calculate_cfg_complexity(graph),
            });
        }
    }

    // Sort by complexity (most complex first)
    graph_entries.sort_by(|a, b| b.complexity.cmp(&a.complexity));

    // Generate HTML
    let html_content = generate_cfg_gallery_html(&graph_entries, generate_images);
    fs::write(output_path, html_content)?;

    Ok(())
}

struct GraphEntry {
    function_name: String,
    svg_path: String,
    nodes: usize,
    edges: usize,
    complexity: usize,
}

fn calculate_cfg_complexity(graph: &cfg::ControlFlowGraph) -> usize {
    // McCabe's cyclomatic complexity: E - N + 2P
    // where E = edges, N = nodes, P = connected components (assume 1)
    if graph.blocks.is_empty() {
        return 1;
    }
    let edge_count = calculate_edge_count(graph);
    edge_count
        .saturating_sub(graph.blocks.len())
        .saturating_add(2)
}

fn calculate_edge_count(graph: &cfg::ControlFlowGraph) -> usize {
    graph
        .blocks
        .iter()
        .map(|(_, block)| block.succs.len())
        .sum()
}

fn generate_cfg_gallery_html(entries: &[GraphEntry], has_images: bool) -> String {
    let graph_cards = entries.iter().map(|entry| {
        let complexity_class = match entry.complexity {
            1..=5 => "complexity-low",
            6..=10 => "complexity-medium",
            _ => "complexity-high",
        };
        
        if has_images {
            format!(
                r#"<div class="graph-card">
                    <div class="graph-header">
                        <h3>{}</h3>
                        <div class="graph-metrics">
                            <span class="metric">Nodes: <strong>{}</strong></span>
                            <span class="metric">Edges: <strong>{}</strong></span>
                            <span class="metric complexity {}">Complexity: <strong>{}</strong></span>
                        </div>
                    </div>
                    <div class="graph-container">
                        <img src="{}" alt="CFG for {}" class="graph-image" onclick="openModal(this)">
                    </div>
                </div>"#,
                entry.function_name, entry.nodes, entry.edges, 
                complexity_class, entry.complexity,
                entry.svg_path, entry.function_name
            )
        } else {
            format!(
                r#"<div class="graph-card">
                    <div class="graph-header">
                        <h3>{}</h3>
                        <div class="graph-metrics">
                            <span class="metric">Nodes: <strong>{}</strong></span>
                            <span class="metric">Edges: <strong>{}</strong></span>
                            <span class="metric complexity {}">Complexity: <strong>{}</strong></span>
                        </div>
                    </div>
                    <div class="dot-container">
                        <a href="{}" download class="dot-link">📄 Download DOT file</a>
                    </div>
                </div>"#,
                entry.function_name, entry.nodes, entry.edges,
                complexity_class, entry.complexity, entry.svg_path
            )
        }
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CFG Gallery - SkanUJkod</title>
    <style>
        body {{ 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        
        .container {{ 
            max-width: 1400px; 
            margin: 0 auto; 
            background: white; 
            border-radius: 12px; 
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        
        .header {{
            background: linear-gradient(135deg, #2c3e50 0%, #3498db 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
            font-size: 1.1em;
        }}
        
        .stats {{
            display: flex;
            justify-content: center;
            gap: 30px;
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid rgba(255,255,255,0.2);
        }}
        
        .stat {{
            text-align: center;
        }}
        
        .stat-number {{
            font-size: 2em;
            font-weight: bold;
            display: block;
        }}
        
        .content {{
            padding: 30px;
        }}
        
        .graphs-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 25px;
            margin-top: 20px;
        }}
        
        .graph-card {{
            border: 1px solid #e0e0e0;
            border-radius: 12px;
            overflow: hidden;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
            background: white;
        }}
        
        .graph-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 12px 25px rgba(0,0,0,0.15);
        }}
        
        .graph-header {{
            padding: 20px;
            background: #f8f9fa;
            border-bottom: 1px solid #e0e0e0;
        }}
        
        .graph-header h3 {{
            margin: 0 0 15px 0;
            color: #2c3e50;
            font-size: 1.3em;
            font-weight: 600;
        }}
        
        .graph-metrics {{
            display: flex;
            gap: 15px;
            flex-wrap: wrap;
        }}
        
        .metric {{
            background: white;
            padding: 8px 12px;
            border-radius: 20px;
            font-size: 0.9em;
            border: 1px solid #e0e0e0;
        }}
        
        .complexity.complexity-low {{ border-color: #27ae60; color: #27ae60; }}
        .complexity.complexity-medium {{ border-color: #f39c12; color: #f39c12; }}
        .complexity.complexity-high {{ border-color: #e74c3c; color: #e74c3c; }}
        
        .graph-container {{
            padding: 20px;
            text-align: center;
            background: #fafafa;
        }}
        
        .graph-image {{
            max-width: 100%;
            height: auto;
            border-radius: 8px;
            cursor: pointer;
            transition: transform 0.2s ease;
        }}
        
        .graph-image:hover {{
            transform: scale(1.02);
        }}
        
        .dot-container {{
            padding: 20px;
            text-align: center;
            background: #fafafa;
        }}
        
        .dot-link {{
            display: inline-block;
            padding: 12px 24px;
            background: #3498db;
            color: white;
            text-decoration: none;
            border-radius: 25px;
            transition: background 0.3s ease;
        }}
        
        .dot-link:hover {{
            background: #2980b9;
        }}
        
        /* Modal for image viewing */
        .modal {{
            display: none;
            position: fixed;
            z-index: 1000;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
            background-color: rgba(0,0,0,0.9);
        }}
        
        .modal-content {{
            margin: 5% auto;
            display: block;
            max-width: 90%;
            max-height: 90%;
        }}
        
        .close {{
            position: absolute;
            top: 15px;
            right: 35px;
            color: #f1f1f1;
            font-size: 40px;
            font-weight: bold;
            cursor: pointer;
        }}
        
        .close:hover {{
            color: #3498db;
        }}
        
        .controls {{
            text-align: center;
            margin-bottom: 30px;
        }}
        
        .filter-button {{
            padding: 10px 20px;
            margin: 0 5px;
            border: none;
            border-radius: 25px;
            background: #ecf0f1;
            cursor: pointer;
            transition: background 0.3s ease;
        }}
        
        .filter-button.active,
        .filter-button:hover {{
            background: #3498db;
            color: white;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔀 Control Flow Graph Gallery</h1>
            <p>Visual representation of program control flow</p>
            <div class="stats">
                <div class="stat">
                    <span class="stat-number">{}</span>
                    <span>Functions</span>
                </div>
                <div class="stat">
                    <span class="stat-number">{}</span>
                    <span>Avg Complexity</span>
                </div>
                <div class="stat">
                    <span class="stat-number">{}</span>
                    <span>Max Complexity</span>
                </div>
            </div>
        </div>
        
        <div class="content">
            <div class="controls">
                <button class="filter-button active" onclick="filterGraphs('all')">All</button>
                <button class="filter-button" onclick="filterGraphs('low')">Low Complexity</button>
                <button class="filter-button" onclick="filterGraphs('medium')">Medium Complexity</button>
                <button class="filter-button" onclick="filterGraphs('high')">High Complexity</button>
            </div>
            
            <div class="graphs-grid">
                {}
            </div>
        </div>
    </div>
    
    <!-- Modal for image viewing -->
    <div id="imageModal" class="modal">
        <span class="close" onclick="closeModal()">&times;</span>
        <img class="modal-content" id="modalImage">
    </div>
    
    <script>
        function openModal(img) {{
            const modal = document.getElementById('imageModal');
            const modalImg = document.getElementById('modalImage');
            modal.style.display = 'block';
            modalImg.src = img.src;
        }}
        
        function closeModal() {{
            document.getElementById('imageModal').style.display = 'none';
        }}
        
        function filterGraphs(complexity) {{
            const cards = document.querySelectorAll('.graph-card');
            const buttons = document.querySelectorAll('.filter-button');
            
            // Update active button
            buttons.forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
            
            // Filter cards
            cards.forEach(card => {{
                if (complexity === 'all') {{
                    card.style.display = 'block';
                }} else {{
                    const hasComplexity = card.querySelector('.complexity-' + complexity);
                    card.style.display = hasComplexity ? 'block' : 'none';
                }}
            }});
        }}
        
        // Close modal when clicking outside image
        window.onclick = function(event) {{
            const modal = document.getElementById('imageModal');
            if (event.target === modal) {{
                closeModal();
            }}
        }}
    </script>
</body>
</html>"#,
        entries.len(),
        if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.complexity).sum::<usize>() as f32 / entries.len() as f32
        },
        entries.iter().map(|e| e.complexity).max().unwrap_or(0),
        graph_cards
    )
}
