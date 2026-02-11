#![windows_subsystem = "windows"]

mod lang;

mod syntax_highlight {
    use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};

    #[derive(Clone, Copy)]
    pub struct Theme {
        pub keyword: Color32,      // let, fn, if, else, loop, break, return
        pub builtin: Color32,      // print, true, false
        pub function: Color32,     // имена функций
        pub string: Color32,       // "строки"
        pub number: Color32,       // 123, -45
        pub comment: Color32,      // // комментарии
        pub operator: Color32,     // + - * / = == != < > && ||
        pub bracket: Color32,      // ( ) { }
        pub variable: Color32,     // переменные
        pub special: Color32,      // use_crate, import, rust, extern_fn
        pub default: Color32,      // остальное
    }

    impl Default for Theme {
        fn default() -> Self {
            Theme {
                keyword: Color32::from_rgb(198, 120, 221),   // фиолетовый
                builtin: Color32::from_rgb(86, 182, 194),    // голубой
                function: Color32::from_rgb(97, 175, 239),   // синий
                string: Color32::from_rgb(152, 195, 121),    // зелёный
                number: Color32::from_rgb(209, 154, 102),    // оранжевый
                comment: Color32::from_rgb(92, 99, 112),     // серый
                operator: Color32::from_rgb(224, 108, 117),  // красный
                bracket: Color32::from_rgb(255, 215, 0),     // золотой
                variable: Color32::from_rgb(224, 224, 224),  // белый
                special: Color32::from_rgb(255, 135, 0),     // ярко-оранжевый
                default: Color32::from_rgb(171, 178, 191),   // светло-серый
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TokenType {
        Keyword,
        Builtin,
        Function,
        String,
        Number,
        Comment,
        Operator,
        Bracket,
        Variable,
        Special,
        Whitespace,
        Default,
    }

    struct Highlighter {
        theme: Theme,
        font: FontId,
    }

    impl Highlighter {
        fn new(theme: Theme, font: FontId) -> Self {
            Highlighter { theme, font }
        }

        fn get_color(&self, token_type: TokenType) -> Color32 {
            match token_type {
                TokenType::Keyword => self.theme.keyword,
                TokenType::Builtin => self.theme.builtin,
                TokenType::Function => self.theme.function,
                TokenType::String => self.theme.string,
                TokenType::Number => self.theme.number,
                TokenType::Comment => self.theme.comment,
                TokenType::Operator => self.theme.operator,
                TokenType::Bracket => self.theme.bracket,
                TokenType::Variable => self.theme.variable,
                TokenType::Special => self.theme.special,
                TokenType::Whitespace => self.theme.default,
                TokenType::Default => self.theme.default,
            }
        }

        fn classify_word(&self, word: &str, prev_token: Option<&str>) -> TokenType {
            match word {
                // Специальные конструкции (RS-- расширения)
                "use_crate" | "import" | "extern_fn" | "rust" => TokenType::Special,

                // Ключевые слова
                "let" | "fn" | "if" | "else" | "loop" | "break" | "return" => TokenType::Keyword,

                // Встроенные
                "print" => TokenType::Builtin,
                "true" | "false" => TokenType::Builtin,
                "int" | "str" | "bool" => TokenType::Builtin,

                // После fn — это имя функции
                _ if prev_token == Some("fn") => TokenType::Function,

                // После extern_fn — тоже имя функции
                _ if prev_token == Some("extern_fn") => TokenType::Function,

                // Остальные идентификаторы — переменные
                _ if word.chars().all(|c| c.is_alphanumeric() || c == '_') 
                     && word.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) 
                => TokenType::Variable,

                _ => TokenType::Default,
            }
        }

        fn tokenize(&self, text: &str) -> Vec<(String, TokenType)> {
            let mut tokens = Vec::new();
            let mut chars = text.chars().peekable();
            let mut prev_word: Option<String> = None;

            while let Some(&ch) = chars.peek() {
                // Комментарии
                if ch == '/' {
                    chars.next();
                    if chars.peek() == Some(&'/') {
                        let mut comment = String::from("/");
                        comment.push(chars.next().unwrap());
                        while let Some(&c) = chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            comment.push(chars.next().unwrap());
                        }
                        tokens.push((comment, TokenType::Comment));
                        prev_word = None;
                        continue;
                    } else {
                        tokens.push(("/".to_string(), TokenType::Operator));
                        continue;
                    }
                }

                // Строки
                if ch == '"' {
                    let mut string = String::new();
                    string.push(chars.next().unwrap());
                    while let Some(&c) = chars.peek() {
                        string.push(chars.next().unwrap());
                        if c == '"' && !string.ends_with("\\\"") {
                            break;
                        }
                        if c == '\\' {
                            // Было: if let Some(&escaped) = chars.peek() {
                            if chars.peek().is_some() {
                                string.push(chars.next().unwrap());
                            }
                        }
                    }
                    tokens.push((string, TokenType::String));
                    prev_word = None;
                    continue;
                }

                // Числа
                if ch.is_ascii_digit() || (ch == '-' && chars.clone().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false)) {
                    let mut num = String::new();
                    if ch == '-' {
                        num.push(chars.next().unwrap());
                    }
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if !num.is_empty() && num != "-" {
                        tokens.push((num, TokenType::Number));
                        prev_word = None;
                        continue;
                    } else if num == "-" {
                        tokens.push((num, TokenType::Operator));
                        continue;
                    }
                }

                // Идентификаторы и ключевые слова
                if ch.is_alphabetic() || ch == '_' {
                    let mut word = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            word.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    let token_type = self.classify_word(&word, prev_word.as_deref());
                    tokens.push((word.clone(), token_type));
                    prev_word = Some(word);
                    continue;
                }

                // Скобки
                if ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' {
                    tokens.push((chars.next().unwrap().to_string(), TokenType::Bracket));
                    continue;
                }

                // Операторы (двухсимвольные)
                if ch == '=' || ch == '!' || ch == '<' || ch == '>' || ch == '&' || ch == '|' {
                    let mut op = chars.next().unwrap().to_string();
                    if let Some(&next) = chars.peek() {
                        if (ch == '=' && next == '=') ||
                           (ch == '!' && next == '=') ||
                           (ch == '<' && next == '=') ||
                           (ch == '>' && next == '=') ||
                           (ch == '&' && next == '&') ||
                           (ch == '|' && next == '|') {
                            op.push(chars.next().unwrap());
                        }
                    }
                    tokens.push((op, TokenType::Operator));
                    continue;
                }

                // Другие операторы
                if ch == '+' || ch == '-' || ch == '*' || ch == '%' || ch == ';' || ch == ':' || ch == ',' {
                    tokens.push((chars.next().unwrap().to_string(), TokenType::Operator));
                    continue;
                }

                // Пробелы и переносы
                if ch.is_whitespace() {
                    let mut ws = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace() {
                            ws.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push((ws, TokenType::Whitespace));
                    continue;
                }

                // Всё остальное
                tokens.push((chars.next().unwrap().to_string(), TokenType::Default));
            }

            tokens
        }

        fn highlight(&self, text: &str) -> LayoutJob {
            let mut job = LayoutJob::default();
            let tokens = self.tokenize(text);

            for (token_text, token_type) in tokens {
                let color = self.get_color(token_type);
                job.append(
                    &token_text,
                    0.0,
                    TextFormat {
                        font_id: self.font.clone(),
                        color,
                        ..Default::default()
                    },
                );
            }

            job
        }
    }

    pub fn highlight_code(text: &str, font_size: f32) -> LayoutJob {
        let theme = Theme::default();
        let font = FontId::monospace(font_size);
        let highlighter = Highlighter::new(theme, font);
        highlighter.highlight(text)
    }

    // Виджет для редактора с подсветкой
	pub fn code_editor(
		ui: &mut egui::Ui,
		code: &mut String,
		id: &str,
		error_lines: &std::collections::HashSet<usize>, // номера строк с ошибками (1-based)
	) -> egui::Response {
		let font_size = 14.0;
		let line_count = code.chars().filter(|c| *c == '\n').count() + 1;

		let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
			let layout_job = highlight_code(text, font_size);
			ui.fonts(|f| f.layout_job(layout_job))
		};

		let mut line_nums = String::new();
		for i in 1..=line_count {
			if error_lines.contains(&i) {
				line_nums.push_str(&format!("⚠{:>3} │\n", i));
			} else {
				line_nums.push_str(&format!("{:>4} │\n", i));
			}
		}

		egui::ScrollArea::both()
			.id_source(id)
			.show(ui, |ui| {
				ui.horizontal_top(|ui| {
					// Номера строк с ошибками
					ui.vertical(|ui| {
						for i in 1..=line_count {
							if error_lines.contains(&i) {
								ui.label(
									egui::RichText::new(format!("⚠{:>3} │", i))
										.monospace()
										.size(font_size)
										.color(egui::Color32::from_rgb(255, 80, 80)),
								);
							} else {
								ui.label(
									egui::RichText::new(format!("{:>4} │", i))
										.monospace()
										.size(font_size)
										.color(egui::Color32::from_rgb(90, 95, 110)),
								);
							}
						}
					});

					ui.add(
						egui::TextEdit::multiline(code)
							.font(egui::FontId::monospace(font_size))
							.code_editor()
							.desired_width(ui.available_width() - 10.0)
							.desired_rows(35)
							.layouter(&mut layouter),
					)
				})
				.inner
			})
			.inner
	}
}

use eframe::egui;
use lang::*;
use std::fs;

fn create_default_project() -> std::path::PathBuf {
    let project_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("rust_minus_minus_project");

    // Создаём папку если нет
    let _ = std::fs::create_dir_all(&project_dir);

    // main.rsmm
    let main_path = project_dir.join("main.rsmm");
    if !main_path.exists() {
        let main_content = r#"// Главный файл Rust--
// Используем библиотеку rand для случайных чисел
use_crate("rand", "0.8");

// Импортируем наш модуль
import("math.rsmm");

// Блок чистого Rust кода
rust {
    use rand::Rng;
}

// Внешняя функция на Rust
extern_fn random(min, max) {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(min.as_int()..=max.as_int());
    RsmmVal::Int(n)
}

let x = 10;
let y = 20;

// Вызываем функцию из math.rsmm
add(x, y);

// Вызываем внешнюю Rust функцию
let r = random(1, 100);
print("Случайное число: " + r);

print("Готово!");
"#;
        let _ = std::fs::write(&main_path, main_content);
    }

    // math.rsmm
    let math_path = project_dir.join("math.rsmm");
    if !math_path.exists() {
        let math_content = r#"// Библиотека математических функций

fn add(a, b) {
    let result = a + b;
    print("Сумма: " + result);
}

fn multiply(a, b) {
    let result = a * b;
    print("Произведение: " + result);
}

fn factorial(n) {
    let result = 1;
    let i = 1;
    loop {
        if i > n {
            break;
        }
        result = result * i;
        i = i + 1;
    }
    print(n + "! = " + result);
}
"#;
        let _ = std::fs::write(&math_path, math_content);
    }

    // utils.rsmm
    let utils_path = project_dir.join("utils.rsmm");
    if !utils_path.exists() {
        let utils_content = r#"// Утилиты для Rust--

fn separator() {
    print("────────────────────────────────");
}

fn repeat_str(s, n) {
    let result = "";
    let i = 0;
    loop {
        if i >= n {
            break;
        }
        result = result + s;
        i = i + 1;
    }
    print(result);
}

fn is_even(n) {
    if n % 2 == 0 {
        print(n + " — чётное");
    } else {
        print(n + " — нечётное");
    }
}
"#;
        let _ = std::fs::write(&utils_path, utils_content);
    }

    project_dir
}

fn main() -> eframe::Result<()> {
    // Загружаем иконку из PNG
    let icon_data = include_bytes!("../IDE.png");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = icon_image.dimensions();
    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Rust-- IDE")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "Rust-- IDE",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(RustMinusMinusApp::new()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    use std::sync::Arc;
    
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "emoji_fallback".to_owned(),
        Arc::new(egui::FontData::from_static(
            include_bytes!("../fonts/NotoEmoji-Regular.ttf")
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("emoji_fallback".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("emoji_fallback".to_owned());

    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}

#[derive(PartialEq)]
enum Tab {
    Compiler,
    Project,
    Decompiler,
}

#[derive(Clone, Copy, PartialEq)]
enum CompileTarget {
    Rust,
    Bytecode,
}

#[derive(Clone, Copy, PartialEq)]
enum ThemeMode {
    Dark,
    Light,
}
#[derive(Clone, Copy, PartialEq)]
enum CompressionLevel {
    None,
    Fast,
    Balanced,
    Max,
}

#[derive(Clone)]
struct Settings {
    language: Language,
    theme_mode: ThemeMode,
    compile_target: CompileTarget,
    compression_level: CompressionLevel,
    project_path: String, // ← НОВОЕ
}

impl Settings {
	fn settings_path() -> std::path::PathBuf {
		std::env::current_exe()
			.ok()
			.and_then(|p| p.parent().map(|p| p.to_path_buf()))
			.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
			.join("ide.settings")
	}

    fn save(&self) {
        let lang = match self.language {
            Language::Ru => "ru",
            Language::En => "en",
        };
        let theme = match self.theme_mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        };
        let target = match self.compile_target {
            CompileTarget::Rust => "rust",
            CompileTarget::Bytecode => "bytecode",
        };
        let compression = match self.compression_level {
            CompressionLevel::None => "none",
            CompressionLevel::Fast => "fast",
            CompressionLevel::Balanced => "balanced",
            CompressionLevel::Max => "max",
        };

        let content = format!(
            "language={}\ntheme={}\ntarget={}\ncompression={}\nproject_path={}\n",
            lang, theme, target, compression, self.project_path
        );

        let _ = std::fs::write(Self::settings_path(), content);
    }

	fn load() -> Self {
		// Дефолтный путь — рядом с exe, НЕ в temp
		let exe_dir = std::env::current_exe()
			.ok()
			.and_then(|p| p.parent().map(|p| p.to_path_buf()))
			.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
		
		let default_project = exe_dir
			.join("rust_minus_minus_project")
			.to_string_lossy()
			.to_string();

		let default = Settings {
			language: Language::Ru,
			theme_mode: ThemeMode::Dark,
			compile_target: CompileTarget::Rust,
			compression_level: CompressionLevel::Balanced,
			project_path: default_project.clone(),
		};

		let path = Self::settings_path();
		let content = match std::fs::read_to_string(&path) {
			Ok(c) => c,
			Err(_) => return default,
		};

		let mut settings = default;

		for line in content.lines() {
			let line = line.trim();
			if let Some((key, value)) = line.split_once('=') {
				let key = key.trim();
				let value = value.trim();
				match key {
					"language" => {
						settings.language = match value {
							"en" => Language::En,
							_ => Language::Ru,
						};
					}
					"theme" => {
						settings.theme_mode = match value {
							"light" => ThemeMode::Light,
							_ => ThemeMode::Dark,
						};
					}
					"target" => {
						settings.compile_target = match value {
							"bytecode" => CompileTarget::Bytecode,
							_ => CompileTarget::Rust,
						};
					}
					"compression" => {
						settings.compression_level = match value {
							"none" => CompressionLevel::None,
							"fast" => CompressionLevel::Fast,
							"max" => CompressionLevel::Max,
							_ => CompressionLevel::Balanced,
						};
					}
					"project_path" => {
						if !value.is_empty() {
							settings.project_path = value.to_string();
						}
					}
					_ => {}
				}
			}
		}

		settings
	}
}

struct RustMinusMinusApp {
    // Tab
    active_tab: Tab,

    // Compiler tab (оставь как было)
    source_code: String,
    compiler_output: String,
    run_output: String,
    bytecode_view: String,
    compiled_bytes: Option<Vec<u8>>,
    show_bytecode: bool,

    // Project tab (НОВОЕ)
    project_files: Vec<ProjectFileUI>,
    active_file_idx: usize,
    project_deps: Vec<DepUI>,
    project_name: String,
    new_file_name: String,
    new_dep_name: String,
    new_dep_version: String,
    new_dep_features: String,

    // Decompiler tab (оставь как было)
    decompiled_source: String,
    decompiler_status: String,
    loaded_file_name: String,

    // Common
    status_msg: String,
    show_guide: bool,
	language: Language,
	show_settings: bool,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    last_saved_code: String,
    project_run_output: String,
    project_compiler_output: String,
    project_compiled_bytes: Option<Vec<u8>>,
    project_output: String,  
    theme_mode: ThemeMode,
    compile_target: CompileTarget,
    compression_level: CompressionLevel,
    error_lines: std::collections::HashSet<usize>,
    project_error_lines: std::collections::HashSet<usize>,
    project_path: String,
    last_file_contents: Vec<String>,
}
#[derive(Clone)]
struct ProjectFileUI {
    name: String,
    content: String,
    is_main: bool,
}

#[derive(Clone)]
struct DepUI {
    name: String,
    version: String,
    features: String,
}
#[derive(Clone, Copy, PartialEq)]
enum Language {
    Ru,
    En,
}

struct Texts {
    // Tabs
    compiler: &'static str,
    project: &'static str,
    decompiler: &'static str,
    guide: &'static str,
    settings: &'static str,

    // Compiler tab
    run: &'static str,
    compile: &'static str,
    save_rsm: &'static str,
    build_exe: &'static str,
    installer_msi: &'static str,
    show_rust: &'static str,
    open: &'static str,
    save_source: &'static str,
    bytecode: &'static str,
    examples: &'static str,
    source_code: &'static str,
    compilation: &'static str,
    program_output: &'static str,

    // Project tab
    build_project_exe: &'static str,
    installer: &'static str,
    show_rust_code: &'static str,
    project_label: &'static str,
    save_files: &'static str,
    project_files: &'static str,
    dependencies: &'static str,
    add_file: &'static str,
    add_crate: &'static str,
    name_label: &'static str,
    version_label: &'static str,
    features_label: &'static str,
    result: &'static str,
    editor: &'static str,
    project_run: &'static str,
    project_compile: &'static str,
    project_save_source: &'static str,
    project_compiler_output: &'static str,
    project_compile_output: &'static str,
    project_label_name: &'static str,
    project_build_btn: &'static str,
    project_installer_btn: &'static str,
    project_show_rust_btn: &'static str,
    new_file_comment: &'static str,
    dep_name: &'static str,
    dep_version: &'static str,
    dep_features: &'static str,

    // Decompiler tab
    load_rsm: &'static str,
    save_as_rsmm: &'static str,
    copy_to_compiler: &'static str,
    decompiled_code: &'static str,

    // Settings
    language: &'static str,
    theme_label: &'static str,

    // Status
    compilation_success: &'static str,
    compilation_error: &'static str,
    execution_done: &'static str,
    execution_error: &'static str,
    saved: &'static str,
    compile_first: &'static str,
    nothing_to_save: &'static str,
    files_saved: &'static str,
    rust_generated: &'static str,
    build_error: &'static str,
    installer_error: &'static str,
    code_copied: &'static str,
    undo: &'static str,
    redo: &'static str,

    // Подробные статусы
    compilation_success_detail: &'static str,
    compilation_error_detail: &'static str,
    compilation_instructions: &'static str,
    compilation_functions_count: &'static str,
    compilation_bytecode_size: &'static str,
    execution_error_detail: &'static str,
    no_output: &'static str,
    status_exe_built: &'static str,
    status_installer_built: &'static str,
    status_error: &'static str,
    status_loaded: &'static str,
    status_save_error: &'static str,
    status_read_error: &'static str,
    status_decompile_success: &'static str,
    status_decompile_error: &'static str,
    status_nothing_to_decompile: &'static str,
    status_rust_code_generated: &'static str,
    copied: &'static str,

    // Заголовки
    ide_title: &'static str,
    version: &'static str,
    guide_title: &'static str,
    settings_title: &'static str,
    settings_description: &'static str,

    // Примеры
    examples_menu: &'static str,
    example_hello: &'static str,
    example_counter: &'static str,
    example_fibonacci: &'static str,
    example_fizzbuzz: &'static str,
    example_functions: &'static str,

    // Редактор
    code_label: &'static str,
    bytecode_view_label: &'static str,
    compile_output_label: &'static str,
    run_output_label: &'static str,

    // Декомпилятор
    decompiler_load_prompt: &'static str,
	dark_mode: &'static str,
    light_mode: &'static str,
    compile_target: &'static str,
    compile_to_rust: &'static str,
    compile_to_bytecode: &'static str,
    build_settings: &'static str,
    compression_level: &'static str,
    compression_none: &'static str,
    compression_fast: &'static str,
    compression_balanced: &'static str,
    compression_max: &'static str,
    compression_hint: &'static str,
	native_compile: &'static str,
	native_compile_beta: &'static str,
    project_path_label: &'static str,
    project_path_hint: &'static str,
    project_path_browse: &'static str,
    project_path_reload: &'static str,
    project_path_changed: &'static str,
}

const TEXTS_RU: Texts = Texts {
    compiler: "🔧 Компилятор",
    project: "📁 Проект",
    decompiler: "🔍 Декомпилятор",
    guide: "📖 Гайд",
    settings: "⚙ Настройки",

    run: "▶ Зап.",
    compile: "🔨 Комп.",
    save_rsm: "💾 Сохранить .rsm",
    build_exe: "🏗 Собрать .exe",
    installer_msi: "📦 Установщик .msi",
    show_rust: "Rust код",
    open: "📂 Открыть",
    save_source: "💾 Сохранить исходник",
    bytecode: "📋 Байткод",
    examples: "📝 Примеры",
    source_code: "📝 Исходный код (.rsmm)",
    compilation: "🔧 Компиляция",
    program_output: "▶ Вывод программы",

    build_project_exe: "🏗 Собрать .exe",
    installer: "📦 Установщик",
    show_rust_code: "Rust код",
    project_label: "Проект:",
    save_files: "💾 Сохранить файлы",
    project_files: "📁 Файлы проекта",
    dependencies: "📦 Зависимости (crates)",
    add_file: "➕",
    add_crate: "➕ Добавить crate",
    name_label: "Имя:",
    version_label: "Версия:",
    features_label: "Фичи:",
    result: "📊 Результат",
    editor: "📝",
    project_run: "▶ Зап.",
    project_compile: "🔨 Комп.",
    project_save_source: "💾 Сохранить исходники",
    project_compiler_output: "▶ Вывод проекта",
    project_compile_output: "🔧 Компиляция проекта",
    project_label_name: "Проект:",
    project_build_btn: "🚀 Собрать .exe",
    project_installer_btn: "📦 Установщик .msi(exe)",
    project_show_rust_btn: "Rust код",
    new_file_comment: "// Новый файл\n",
    dep_name: "Имя:",
    dep_version: "Версия:",
    dep_features: "Фичи:",

    load_rsm: "📂 Загрузить .rsm",
    save_as_rsmm: "💾 Сохранить как .rsmm",
    copy_to_compiler: "📋 Копировать в компилятор",
    decompiled_code: "📝 Декомпилированный код Rust--",

    language: "Язык / Language",
    theme_label: "Тема",

    compilation_success: "Компиляция завершена успешно",
    compilation_error: "Ошибка компиляции",
    execution_done: "Выполнение завершено",
    execution_error: "Ошибка выполнения",
    saved: "Сохранено",
    compile_first: "Сначала скомпилируйте код",
    nothing_to_save: "Нечего сохранять",
    files_saved: "Файлы проекта сохранены на диск",
    rust_generated: "Rust код сгенерирован",
    build_error: "Ошибка сборки",
    installer_error: "Ошибка создания установщика",
    code_copied: "Код скопирован в компилятор",
    undo: "↩ Отменить",
    redo: "↪ Повторить",

    compilation_success_detail: "✅ Компиляция успешна!",
    compilation_error_detail: "❌ Ошибка компиляции",
    compilation_instructions: "Инструкций",
    compilation_functions_count: "Функций",
    compilation_bytecode_size: "Размер байткода (байт)",
    execution_error_detail: "❌ Ошибка выполнения",
    no_output: "(нет вывода)",
    status_exe_built: "✅ EXE собран",
    status_installer_built: "✅ Установщик создан",
    status_error: "Ошибка",
    status_loaded: "Загружен",
    status_save_error: "Ошибка сохранения",
    status_read_error: "❌ Ошибка чтения файла",
    status_decompile_success: "✅ Декомпиляция успешна! Файл",
    status_decompile_error: "❌ Ошибка декомпиляции",
    status_nothing_to_decompile: "Нечего сохранять",
    status_rust_code_generated: "Rust код сгенерирован",
    copied: "Код скопирован!",

    ide_title: "🦀 Rust-- IDE",
    version: "Rust-- v1.0",
    guide_title: "📚 Полный гайд по Rust--",
    settings_title: "⚙ Настройки",
    settings_description: "Максимально упрощённый Rust",

    examples_menu: "📚 Примеры",
    example_hello: "Привет мир",
    example_counter: "Счётчик",
    example_fibonacci: "Фибоначчи",
    example_fizzbuzz: "FizzBuzz",
    example_functions: "Функции",

    code_label: "📝 Исходный код (.rsmm)",
    bytecode_view_label: "📋 Байткод",
    compile_output_label: "🔨 Компиляция",
    run_output_label: "▶ Вывод программы",

    decompiler_load_prompt: "Загрузите .rsm файл для декомпиляции",
	dark_mode: "🌙 Тёмная тема",
    light_mode: "☀️ Светлая тема",
    compile_target: "Целевой код",
    compile_to_rust: "🦀 Rust (нативный .exe)",
    compile_to_bytecode: "⚡ Байткод RS-- (интерпретатор)",
    build_settings: "Настройки сборки",
    compression_level: "Сжатие .exe / Installer",
    compression_none: "🔓 Без сжатия (быстрая сборка)",
    compression_fast: "⚡ Быстрое сжатие",
    compression_balanced: "⚖️ Сбалансированное",
    compression_max: "📦 Максимальное (медленно, минимальный размер)",
    compression_hint: "💡 Максимальное сжатие уменьшает размер на 30-50%, но сборка дольше",
	native_compile: "⚡ Нативная компиляция (BETA)",
	native_compile_beta: "Прямая компиляция в машинный код без Rust",
    project_path_label: "📁 Путь к проекту",
    project_path_hint: "Папка где хранятся файлы .rsmm",
    project_path_browse: "📂 Обзор...",
    project_path_reload: "🔄 Перечитать файлы",
    project_path_changed: "✅ Путь к проекту изменён",
};

const TEXTS_EN: Texts = Texts {
    compiler: "🔧 Compiler",
    project: "📁 Project",
    decompiler: "🔍 Decompiler",
    guide: "📖 Guide",
    settings: "⚙ Settings",

    run: "▶ Run",
    compile: "🔨 Compile",
    save_rsm: "💾 Save .rsm",
    build_exe: "🏗 Build .exe",
    installer_msi: "📦 Installer .msi",
    show_rust: "Rust code",
    open: "📂 Open",
    save_source: "💾 Save source",
    bytecode: "📋 Bytecode",
    examples: "📝 Examples",
    source_code: "📝 Source code (.rsmm)",
    compilation: "🔧 Compilation",
    program_output: "▶ Program output",

    build_project_exe: "🏗 Build .exe",
    installer: "📦 Installer",
    show_rust_code: "🦀 Show Rust code",
    project_label: "Project:",
    save_files: "💾 Save files",
    project_files: "📁 Project files",
    dependencies: "📦 Dependencies (crates)",
    add_file: "➕",
    add_crate: "➕ Add crate",
    name_label: "Name:",
    version_label: "Version:",
    features_label: "Features:",
    result: "📊 Result",
    editor: "📝",
    project_run: "▶ Run",
    project_compile: "🔨 Compile",
    project_save_source: "💾 Save sources",
    project_compiler_output: "▶ Project output",
    project_compile_output: "🔧 Project compilation",
    project_label_name: "Project:",
    project_build_btn: "🚀 Build .exe",
    project_installer_btn: "📦 Installer .msi(exe)",
    project_show_rust_btn: "Rust code",
    new_file_comment: "// New file\n",
    dep_name: "Name:",
    dep_version: "Version:",
    dep_features: "Features:",

    load_rsm: "📂 Load .rsm",
    save_as_rsmm: "💾 Save as .rsmm",
    copy_to_compiler: "📋 Copy to compiler",
    decompiled_code: "📝 Decompiled Rust-- code",

    language: "Language / Язык",
    theme_label: "Theme",

    compilation_success: "Compilation successful",
    compilation_error: "Compilation error",
    execution_done: "Execution complete",
    execution_error: "Execution error",
    saved: "Saved",
    compile_first: "Compile the code first",
    nothing_to_save: "Nothing to save",
    files_saved: "Project files saved to disk",
    rust_generated: "Rust code generated",
    build_error: "Build error",
    installer_error: "Installer creation error",
    code_copied: "Code copied to compiler",
    undo: "↩ Undo",
    redo: "↪ Redo",

    compilation_success_detail: "✅ Compilation successful!",
    compilation_error_detail: "❌ Compilation error",
    compilation_instructions: "Instructions",
    compilation_functions_count: "Functions",
    compilation_bytecode_size: "Bytecode size (bytes)",
    execution_error_detail: "❌ Execution error",
    no_output: "(no output)",
    status_exe_built: "✅ EXE built",
    status_installer_built: "✅ Installer created",
    status_error: "Error",
    status_loaded: "Loaded",
    status_save_error: "Save error",
    status_read_error: "❌ File read error",
    status_decompile_success: "✅ Decompilation successful! File",
    status_decompile_error: "❌ Decompilation error",
    status_nothing_to_decompile: "Nothing to save",
    status_rust_code_generated: "Rust code generated",
    copied: "Code copied!",

    ide_title: "🦀 Rust-- IDE",
    version: "Rust-- v1.0",
    guide_title: "📚 Full Rust-- Guide",
    settings_title: "⚙ Settings",
    settings_description: "Maximally simplified Rust",

    examples_menu: "📚 Examples",
    example_hello: "Hello World",
    example_counter: "Counter",
    example_fibonacci: "Fibonacci",
    example_fizzbuzz: "FizzBuzz",
    example_functions: "Functions",

    code_label: "📝 Source code (.rsmm)",
    bytecode_view_label: "📋 Bytecode",
    compile_output_label: "🔨 Compilation",
    run_output_label: "▶ Program output",

    decompiler_load_prompt: "Load .rsm file to decompile",
    dark_mode: "🌙 Dark theme",
    light_mode: "☀️ Light theme",
    compile_target: "Compile target",
    compile_to_rust: "🦀 Rust (native .exe)",
    compile_to_bytecode: "⚡ RS-- Bytecode (interpreter)",
    build_settings: "Build settings",
    compression_level: "EXE / Installer compression",
    compression_none: "🔓 No compression (fast build)",
    compression_fast: "⚡ Fast compression",
    compression_balanced: "⚖️ Balanced",
    compression_max: "📦 Maximum (slow, smallest size)",
    compression_hint: "💡 Maximum compression reduces size by 30-50%, but takes longer",
	native_compile: "⚡ Native compile (BETA)",
	native_compile_beta: "Direct compilation to machine code without Rust",
    project_path_label: "📁 Project path",
    project_path_hint: "Folder where .rsmm files are stored",
    project_path_browse: "📂 Browse...",
    project_path_reload: "🔄 Reload files",
    project_path_changed: "✅ Project path changed",
};

impl RustMinusMinusApp {
	fn new() -> Self {
        let settings = Settings::load();
        
        // Используем путь из настроек
        let project_dir = std::path::PathBuf::from(&settings.project_path);
        
        // Создаём папку если нет
        let _ = std::fs::create_dir_all(&project_dir);

        // main.rsmm — создаём дефолтный если нет
        let main_path = project_dir.join("main.rsmm");
        if !main_path.exists() {
            let main_content = r#"// Главный файл Rust--
use_crate("rand", "0.8");
import("math.rsmm");

rust {
    use rand :: Rng ;
}

extern_fn random(min, max) {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(min.as_int()..=max.as_int());
    RsmmVal::Int(n)
}

let x = 10;
let y = 20;
add(x, y);

let r = random(1, 100);
print("Случайное число: " + r);
print("Готово!");
"#;
            let _ = std::fs::write(&main_path, main_content);
        }

        // math.rsmm
        let math_path = project_dir.join("math.rsmm");
        if !math_path.exists() {
            let math_content = r#"// Библиотека математических функций

fn add(a, b) {
    let result = a + b;
    print("Сумма: " + result);
}

fn multiply(a, b) {
    let result = a * b;
    print("Произведение: " + result);
}

fn factorial(n) {
    let result = 1;
    let i = 1;
    loop {
        if i > n {
            break;
        }
        result = result * i;
        i = i + 1;
    }
    print(n + "! = " + result);
}
"#;
            let _ = std::fs::write(&math_path, math_content);
        }

        // utils.rsmm
        let utils_path = project_dir.join("utils.rsmm");
        if !utils_path.exists() {
            let utils_content = r#"// Утилиты для Rust--

fn separator() {
    print("────────────────────────────────");
}

fn repeat_str(s, n) {
    let result = "";
    let i = 0;
    loop {
        if i >= n {
            break;
        }
        result = result + s;
        i = i + 1;
    }
    print(result);
}

fn is_even(n) {
    if n % 2 == 0 {
        print(n + " — чётное");
    } else {
        print(n + " — нечётное");
    }
}
"#;
            let _ = std::fs::write(&utils_path, utils_content);
        }

        let example = r#"// Привет от Rust--!
let name = "World";
let x = 10;
let y = 20;
let sum = x + y;
print("Hello, Rust--!");
print("Сумма: " + sum);

if sum > 25 {
    print("Сумма больше 25!");
} else {
    print("Сумма не больше 25");
}

let i = 0;
loop {
    if i >= 5 {
        break;
    }
    print("Итерация: " + i);
    i = i + 1;
}

fn add(a, b) {
    let result = a + b;
    print("Результат сложения: " + result);
}

add(100, 200);
print("Готово!");
"#;

		let main_content = std::fs::read_to_string(project_dir.join("main.rsmm"))
			.unwrap_or_else(|_| "// main.rsmm\nprint(\"Hello!\");\n".to_string());
		let math_content = std::fs::read_to_string(project_dir.join("math.rsmm"))
			.unwrap_or_else(|_| "// math.rsmm\n".to_string());
		let utils_content = std::fs::read_to_string(project_dir.join("utils.rsmm"))
			.unwrap_or_else(|_| "// utils.rsmm\n".to_string());

		let main_copy = main_content.clone();
		let math_copy = math_content.clone();
		let utils_copy = utils_content.clone();

		let main_file = ProjectFileUI {
			name: "main.rsmm".to_string(),
			content: main_content,
			is_main: true,
		};

		let math_file = ProjectFileUI {
			name: "math.rsmm".to_string(),
			content: math_content,
			is_main: false,
		};

		let utils_file = ProjectFileUI {
			name: "utils.rsmm".to_string(),
			content: utils_content,
			is_main: false,
		};

		let project_dir_str = project_dir.to_string_lossy().to_string();
		
        RustMinusMinusApp {
            active_tab: Tab::Compiler,

            source_code: example.to_string(),
            compiler_output: String::new(),
            run_output: String::new(),
            bytecode_view: String::new(),
            compiled_bytes: None,
            show_bytecode: false,

            project_files: vec![main_file, math_file, utils_file],
            active_file_idx: 0,
            project_deps: vec![
                DepUI {
                    name: "rand".into(),
                    version: "0.8".into(),
                    features: String::new(),
                },
            ],
            project_name: "my_project".to_string(),
            new_file_name: String::new(),
            new_dep_name: String::new(),
            new_dep_version: String::new(),
            new_dep_features: String::new(),

            decompiled_source: String::new(),
            decompiler_status: "Load .rsm to decompile!".into(),
            loaded_file_name: String::new(),

            status_msg: format!("Project: {}", project_dir_str),
            show_guide: false,
            language: settings.language,
            show_settings: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_saved_code: example.to_string(),
            project_run_output: String::new(),
            project_compiler_output: String::new(),
            project_compiled_bytes: None,
            project_output: String::new(),
            theme_mode: settings.theme_mode,
            compile_target: settings.compile_target,
            compression_level: settings.compression_level,
            error_lines: std::collections::HashSet::new(),
            project_error_lines: std::collections::HashSet::new(),
            project_path: settings.project_path, // ← НОВОЕ
			last_file_contents: vec![
				main_copy,
				math_copy, 
				utils_copy,
			],
        }
    }

	fn compile(&mut self) {
		self.compiler_output.clear();
		self.bytecode_view.clear();
		self.run_output.clear(); // ← ДОБАВЛЕНО: очищаем вывод при перекомпиляции
		self.error_lines.clear();

		match self.compile_target {
			CompileTarget::Bytecode => {
				match compile_source(&self.source_code) {
					Ok((program, bytes)) => {
						self.compiled_bytes = Some(bytes);
						self.bytecode_view = disassemble(&program);
						let t = self.texts();
						self.compiler_output = format!(
							"{}\n{}: {}\n{}: {}\n{}: {}",
							t.compilation_success_detail,
							t.compilation_instructions, program.instructions.len(),
							t.compilation_functions_count, program.functions.len(),
							t.compilation_bytecode_size, self.compiled_bytes.as_ref().unwrap().len()
						);
						self.status_msg = t.compilation_success.to_string();
						self.error_lines.clear();
					}
					Err(e) => {
						self.compiled_bytes = None;
						let t = self.texts();
						self.compiler_output = format!("{}\n{}", t.compilation_error_detail, e);
						self.status_msg = t.compilation_error.to_string();
						self.error_lines = Self::parse_error_lines(&self.source_code, &e);
					}
				}
			}
			CompileTarget::Rust => {
				match transpile_to_rust(&self.source_code) {
					Ok(rust_code) => {
						self.bytecode_view = rust_code.clone();
						self.show_bytecode = true;
						let t = self.texts();
						let lines = rust_code.lines().count();
						self.compiler_output = format!(
							"✅ successful!\nLines of code: {}\n\nTo build .exe press: \"🏗 Build .exe\"",
							lines
						);
						self.status_msg = t.rust_generated.to_string();
						self.error_lines.clear();
					}
					Err(e) => {
						let t = self.texts();
						self.compiler_output = format!("{}\n{}", t.compilation_error_detail, e);
						self.status_msg = t.compilation_error.to_string();
						self.error_lines = Self::parse_error_lines(&self.source_code, &e);
					}
				}
			}
		}
	}
	fn get_cargo_profile(&self) -> &'static str {
		match self.compression_level {
			CompressionLevel::None => r#"[profile.release]
	opt-level = 0
	lto = false
	"#,
			CompressionLevel::Fast => r#"[profile.release]
	opt-level = 2
	lto = "thin"
	"#,
			CompressionLevel::Balanced => r#"[profile.release]
	opt-level = "z"
	lto = true
	strip = true
	"#,
			CompressionLevel::Max => r#"[profile.release]
	opt-level = "z"
	lto = true
	strip = true
	codegen-units = 1
	panic = "abort"
	"#,
		}
	}

	fn run(&mut self) {
		self.run_output.clear();
		self.error_lines.clear();

		match self.compile_target {
			CompileTarget::Bytecode => {
				match run_source(&self.source_code) {
					Ok(output) => {
						let t = self.texts();
						if output.is_empty() {
							self.run_output = t.no_output.to_string();
						} else {
							self.run_output = output.join("\n");
						}
						self.status_msg = t.execution_done.to_string();
						self.error_lines.clear();
					}
					Err(e) => {
						let t = self.texts();
						self.run_output = format!("{}\n{}", t.execution_error_detail, e);
						self.status_msg = t.execution_error.to_string();
						self.error_lines = Self::parse_error_lines(&self.source_code, &e);
					}
				}
			}
			CompileTarget::Rust => {
				let t = self.texts();
				
				// Создаём exe В ПАПКЕ ПРОЕКТА, не в temp
				let project_dir = std::path::PathBuf::from(&self.project_path);
				let _ = std::fs::create_dir_all(&project_dir);
				let temp_exe = project_dir.join(".rsmm_temp_run.exe");

				match build_exe_v2(&self.source_code, &temp_exe) {
					Ok(_) => {
						// Запускаем С РАБОЧИМ КАТАЛОГОМ = папка проекта
						match std::process::Command::new(&temp_exe)
							.current_dir(&project_dir)
							.output()
						{
							Ok(output) => {
								let stdout = String::from_utf8_lossy(&output.stdout);
								let stderr = String::from_utf8_lossy(&output.stderr);

								if stdout.is_empty() && stderr.is_empty() {
									self.run_output = t.no_output.to_string();
								} else {
									let mut result = String::new();
									if !stdout.is_empty() {
										result.push_str(&stdout);
									}
									if !stderr.is_empty() {
										if !result.is_empty() {
											result.push_str("\n--- stderr ---\n");
										}
										result.push_str(&stderr);
									}
									self.run_output = result;
								}
								self.status_msg = t.execution_done.to_string();
								self.error_lines.clear();
							}
							Err(e) => {
								self.run_output = format!("{}\n{}", t.execution_error_detail, e);
								self.status_msg = t.execution_error.to_string();
								self.error_lines =
									Self::parse_error_lines(&self.source_code, &e.to_string());
							}
						}
						// Удаляем временный файл
						let _ = std::fs::remove_file(&temp_exe);
					}
					Err(e) => {
						self.run_output = format!("{}\n{}", t.compilation_error_detail, e);
						self.status_msg = t.compilation_error.to_string();
						self.error_lines = Self::parse_error_lines(&self.source_code, &e);
					}
				}
			}
		}
	}

    fn save_compiled(&mut self) {
        if let Some(bytes) = &self.compiled_bytes {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Rust-- Binary", &["rsm"])
                .set_file_name("program.rsm")
                .save_file()
            {
                match fs::write(&path, bytes) {
                    Ok(_) => {
                        self.status_msg =
                            format!("Saved: {}", path.display());
                    }
                    Err(e) => {
                        self.status_msg =
                            format!("Err: {}", e);
						self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                    }
                }
            }
        } else {
            self.status_msg = "Compile code first".into();
        }
    }

    fn load_for_decompile(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust-- Binary", &["rsm"])
            .pick_file()
        {
            match fs::read(&path) {
                Ok(bytes) => {
                    self.loaded_file_name =
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                    match decompile_bytes(&bytes) {
                        Ok(source) => {
                            self.decompiled_source = source;
                            self.decompiler_status = format!(
                                "✅ Decompiled! File: {}",
                                self.loaded_file_name
                            );
                        }
                        Err(e) => {
                            self.decompiler_status =
                                format!("❌ Err: {}", e);
							self.error_lines = Self::parse_error_lines(&self.source_code, &e);
                        }
                    }
                }
                Err(e) => {
                    self.decompiler_status =
                        format!("❌ Error reading file: {}", e);
					self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                }
            }
        }
    }

    fn save_decompiled(&mut self) {
        if self.decompiled_source.is_empty() {
            self.decompiler_status = "Nothing to save".into();
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust-- Source", &["rsmm"])
            .set_file_name("decompiled.rsmm")
            .save_file()
        {
            match fs::write(&path, &self.decompiled_source) {
                Ok(_) => {
                    self.decompiler_status =
                        format!("Saved: {}", path.display());
                }
                Err(e) => {
                    self.decompiler_status =
                        format!("Err: {}", e);
					self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                }
            }
        }
    }

    fn load_source(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust-- Source", &["rsmm", "txt"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.source_code = content;
                    self.status_msg = format!("Loaded: {}", path.display());
                }
                Err(e) => {
                    self.status_msg = format!("Err: {}", e);
					self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                }
            }
        }
    }

    fn save_source(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust-- Source", &["rsmm"])
            .set_file_name("program.rsmm")
            .save_file()
        {
            match fs::write(&path, &self.source_code) {
                Ok(_) => {
                    self.status_msg = format!("Saved: {}", path.display());
                }
                Err(e) => {
                    self.status_msg = format!("Err: {}", e);
					self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                }
            }
        }
    }
}

impl eframe::App for RustMinusMinusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
		
        match self.theme_mode {
            ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
        }

        // Снимок для undo при изменении кода
        self.save_undo_snapshot();
		if self.active_tab == Tab::Project {
			self.auto_save_changed_files();
		}

        // Горячие клавиши Ctrl+Z / Ctrl+Y
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.redo();
        }
		if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
			match self.active_tab {
				Tab::Compiler => {
					self.save_source();
				}
				Tab::Project => {
					self.save_project_files();
					self.status_msg = self.texts().files_saved.to_string();
				}
				Tab::Decompiler => {
					if !self.decompiled_source.is_empty() {
						self.save_decompiled();
					}
				}
			}
		}

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🦀 Rust-- IDE");
                ui.separator();

                if ui
                    .selectable_label(self.active_tab == Tab::Compiler, self.texts().compiler)
                    .clicked()
                {
                    self.active_tab = Tab::Compiler;
                }
                // НОВЫЙ ТАБ:
                if ui
                    .selectable_label(self.active_tab == Tab::Project, self.texts().project)
                    .clicked()
                {
                    self.active_tab = Tab::Project;
                }
                if ui
                    .selectable_label(self.active_tab == Tab::Decompiler, self.texts().decompiler)
                    .clicked()
                {
                    self.active_tab = Tab::Decompiler;
                }
				ui.separator();
        
				// КНОПКА ГАЙДА
				if ui.button(self.texts().guide).clicked() {
					self.show_guide = true;
				}
				if ui.button(self.texts().settings).clicked() {
					self.show_settings = true;
				}

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Rust-- v1.0")
                            .color(egui::Color32::from_rgb(100, 150, 255))
                            .italics(),
                    );
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("📌");
                ui.label(&self.status_msg);
            });
        });
		
		self.render_guide_window(ctx);
		self.render_settings_window(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Compiler => self.render_compiler_tab(ui),
                Tab::Project => self.render_project_tab(ui),   // НОВЫЙ
                Tab::Decompiler => self.render_decompiler_tab(ui),
            }
        });
    }
}

impl RustMinusMinusApp {
	fn texts(&self) -> &'static Texts {
		match self.language {
			Language::Ru => &TEXTS_RU,
			Language::En => &TEXTS_EN,
		}
	}
	fn save_single_file(&self, file: &ProjectFileUI) {
		let project_dir = std::path::PathBuf::from(&self.project_path);
		let _ = std::fs::create_dir_all(&project_dir);
		let path = project_dir.join(&file.name);
		let _ = std::fs::write(&path, &file.content);
	}

	/// Удаляет файл с диска
	fn delete_file_from_disk(&self, filename: &str) {
		let project_dir = std::path::PathBuf::from(&self.project_path);
		let path = project_dir.join(filename);
		let _ = std::fs::remove_file(&path);
	}

	/// Проверяет изменения и автосохраняет
	fn auto_save_changed_files(&mut self) {
		// Расширяем массив если нужно
		while self.last_file_contents.len() < self.project_files.len() {
			self.last_file_contents.push(String::new());
		}

		for i in 0..self.project_files.len() {
			if i < self.last_file_contents.len() {
				if self.project_files[i].content != self.last_file_contents[i] {
					// Содержимое изменилось — сохраняем на диск
					self.save_single_file(&self.project_files[i]);
					self.last_file_contents[i] = self.project_files[i].content.clone();
				}
			}
		}

		// Обрезаем если файлов стало меньше
		self.last_file_contents.truncate(self.project_files.len());
	}

	/// Создаёт новый файл и сразу сохраняет на диск
	fn create_new_file(&mut self, name: String, content: String, is_main: bool) {
		let file = ProjectFileUI {
			name: name.clone(),
			content: content.clone(),
			is_main,
		};

		// Сохраняем на диск СРАЗУ
		self.save_single_file(&file);

		// Добавляем в список
		self.project_files.push(file);
		self.last_file_contents.push(content);
		self.active_file_idx = self.project_files.len() - 1;

		self.status_msg = format!("📄 Created: {}", name);
	}

	/// Удаляет файл из проекта и с диска
	fn delete_project_file(&mut self, idx: usize) {
		if idx < self.project_files.len() && !self.project_files[idx].is_main {
			let filename = self.project_files[idx].name.clone();

			// Удаляем с диска
			self.delete_file_from_disk(&filename);

			// Удаляем из массива
			self.project_files.remove(idx);
			if idx < self.last_file_contents.len() {
				self.last_file_contents.remove(idx);
			}

			// Корректируем индекс
			if self.active_file_idx >= self.project_files.len() {
				self.active_file_idx = self.project_files.len().saturating_sub(1);
			}

			self.status_msg = format!("🗑 Deleted: {}", filename);
		}
	}
	fn reload_project_files(&mut self) {
		let project_dir = std::path::PathBuf::from(&self.project_path);
		self.last_file_contents = self.project_files.iter()
			.map(|f| f.content.clone())
			.collect();
			
		if !project_dir.exists() {
			let _ = std::fs::create_dir_all(&project_dir);
			self.status_msg = format!("📁 Created: {}", self.project_path);
			return;
		}
		
		// Собираем все .rsmm файлы из папки
		let mut new_files: Vec<ProjectFileUI> = Vec::new();
		
		if let Ok(entries) = std::fs::read_dir(&project_dir) {
			let mut file_entries: Vec<_> = entries
				.flatten()
				.filter(|e| {
					e.file_name()
						.to_string_lossy()
						.ends_with(".rsmm")
				})
				.collect();
			
			// Сортируем: main.rsmm первым
			file_entries.sort_by(|a, b| {
				let a_name = a.file_name().to_string_lossy().to_string();
				let b_name = b.file_name().to_string_lossy().to_string();
				if a_name == "main.rsmm" {
					std::cmp::Ordering::Less
				} else if b_name == "main.rsmm" {
					std::cmp::Ordering::Greater
				} else {
					a_name.cmp(&b_name)
				}
			});
			
			for entry in file_entries {
				let name = entry.file_name().to_string_lossy().to_string();
				let content = std::fs::read_to_string(entry.path())
					.unwrap_or_else(|_| format!("// {}\n", name));
				let is_main = name == "main.rsmm";
				
				new_files.push(ProjectFileUI {
					name,
					content,
					is_main,
				});
			}
		}
		
		if new_files.is_empty() {
			// Если папка пустая — создаём дефолтный main.rsmm
			let default_main = ProjectFileUI {
				name: "main.rsmm".to_string(),
				content: "// Main file\nprint(\"Hello from Rust--!\");\n".to_string(),
				is_main: true,
			};
			let main_path = project_dir.join("main.rsmm");
			let _ = std::fs::write(&main_path, &default_main.content);
			new_files.push(default_main);
		}
		
		// Если нет main — помечаем первый файл
		if !new_files.iter().any(|f| f.is_main) {
			if let Some(first) = new_files.first_mut() {
				first.is_main = true;
			}
		}
		
		let count = new_files.len();
		self.project_files = new_files;
		self.active_file_idx = 0;
		
		// Пытаемся прочитать зависимости из main.rsmm
		self.project_deps.clear();
		if let Some(main_file) = self.project_files.iter().find(|f| f.is_main) {
			for line in main_file.content.lines() {
				let trimmed = line.trim();
				if trimmed.starts_with("use_crate(") {
					// Парсим use_crate("name", "version", "feature1", "feature2");
					let inner = trimmed
						.trim_start_matches("use_crate(")
						.trim_end_matches(");")
						.trim_end_matches(')');
					
					let parts: Vec<&str> = inner.split(',')
						.map(|s| s.trim().trim_matches('"').trim_matches('\''))
						.collect();
					
					if parts.len() >= 2 {
						let name = parts[0].to_string();
						let version = parts[1].to_string();
						let features = if parts.len() > 2 {
							parts[2..].join(", ")
						} else {
							String::new()
						};
						
						// Проверяем что такой зависимости ещё нет
						if !self.project_deps.iter().any(|d| d.name == name) {
							self.project_deps.push(DepUI {
								name,
								version,
								features,
							});
						}
					}
				}
			}
		}
		
		self.status_msg = format!(
			"🔄 Loaded {} files from {}",
			count, self.project_path
		);
	}
	fn parse_error_lines(source: &str, error: &str) -> std::collections::HashSet<usize> {
		let mut result = std::collections::HashSet::new();
		let error_lower = error.to_lowercase();

		// 1. Ищем номера строк в тексте ошибки
		for pattern in &[
			"строка ", "строке ", "line ", "at line ",
			"позиция ", "position ", "row ",
		] {
			if let Some(pos) = error_lower.find(pattern) {
				let after = &error[pos + pattern.len()..];
				let num_str: String = after
					.chars()
					.skip_while(|c| !c.is_ascii_digit())
					.take_while(|c| c.is_ascii_digit())
					.collect();
				if let Ok(n) = num_str.parse::<usize>() {
					if n > 0 && n < 100000 {
						result.insert(n);
					}
				}
			}
		}

		// 2. Если нашли — возвращаем
		if !result.is_empty() {
			return result;
		}

		// 3. Ищем слова в кавычках из ошибки в исходном коде
		let mut quoted: Vec<String> = Vec::new();
		let mut in_q = false;
		let mut cur = String::new();
		for ch in error.chars() {
			if ch == '\'' || ch == '"' || ch == '`' {
				if in_q {
					if cur.len() >= 2 {
						quoted.push(cur.clone());
					}
					cur.clear();
					in_q = false;
				} else {
					in_q = true;
				}
			} else if in_q {
				cur.push(ch);
			}
		}

		// Ищем ключевые слова из ошибки
		for keyword in &[
			"неожиданный", "unexpected", "ожидался", "expected",
			"не найден", "not found", "undefined", "неизвестн",
		] {
			if error_lower.contains(keyword) {
				// Извлекаем слово после ключевого
				if let Some(pos) = error_lower.find(keyword) {
					let after = &error[pos + keyword.len()..];
					let word: String = after
						.chars()
						.skip_while(|c| !c.is_alphanumeric() && *c != '_')
						.take_while(|c| c.is_alphanumeric() || *c == '_')
						.collect();
					if word.len() >= 2 {
						quoted.push(word);
					}
				}
			}
		}

		for (line_num, line) in source.lines().enumerate() {
			for word in &quoted {
				if line.contains(word.as_str()) {
					result.insert(line_num + 1);
				}
			}
		}

		// 4. Если всё ещё пусто и есть ошибка — помечаем последнюю строку
		if result.is_empty() {
			let has_error = error_lower.contains("ошибка")
				|| error_lower.contains("error")
				|| error_lower.contains("❌")
				|| error_lower.contains("неожиданный")
				|| error_lower.contains("unexpected");
			if has_error {
				let last = source.lines().count();
				if last > 0 {
					result.insert(last);
				}
			}
		}

		result
	}
	fn save_settings(&self) {
		let settings = Settings {
			language: self.language,
			theme_mode: self.theme_mode,
			compile_target: self.compile_target,
			compression_level: self.compression_level,
			project_path: self.project_path.clone(), // ← НОВОЕ
		};
		settings.save();
	}
	fn collect_project_source(&self) -> String {
        let mut full_source = String::new();

        for file in &self.project_files {
            if !file.is_main {
                let filtered = Self::filter_for_interpreter(&file.content);
                full_source.push_str(&filtered);
                full_source.push('\n');
            }
        }

        for file in &self.project_files {
            if file.is_main {
                let filtered = Self::filter_for_interpreter(&file.content);
                full_source.push_str(&filtered);
                full_source.push('\n');
            }
        }

        full_source
    }

    fn filter_for_interpreter(source: &str) -> String {
        let mut result = String::new();
        let mut skip_block = false;
        let mut brace_depth: i32 = 0;
        let mut extern_fn_names: Vec<String> = Vec::new();
        let mut extern_vars: Vec<String> = Vec::new();

        // Первый проход: собираем имена extern_fn
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("extern_fn ") {
                let after = trimmed.strip_prefix("extern_fn ").unwrap().trim();
                if let Some(paren_pos) = after.find('(') {
                    let name = after[..paren_pos].trim().to_string();
                    extern_fn_names.push(name);
                }
            }
        }

        // Второй проход: фильтруем
        for line in source.lines() {
            let trimmed = line.trim();

            if skip_block {
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            skip_block = false;
                            break;
                        }
                    }
                }
                continue;
            }

            if trimmed.starts_with("use_crate(") || trimmed.starts_with("use_crate (") {
                continue;
            }

            if trimmed.starts_with("import(") || trimmed.starts_with("import (") {
                continue;
            }

            if trimmed.starts_with("rust ") || trimmed.starts_with("rust{") {
                skip_block = true;
                brace_depth = 0;
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            skip_block = false;
                            break;
                        }
                    }
                }
                continue;
            }

            if trimmed.starts_with("extern_fn ") {
                skip_block = true;
                brace_depth = 0;
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            skip_block = false;
                            break;
                        }
                    }
                }
                continue;
            }

            // Проверяем вызовы extern функций
            let mut is_extern_call = false;
            for name in &extern_fn_names {
                if trimmed.contains(&format!("{}(", name)) {
                    is_extern_call = true;
                    break;
                }
            }

            if is_extern_call {
                if trimmed.starts_with("let ") {
                    if let Some(eq_pos) = trimmed.find('=') {
                        let var_name = trimmed[4..eq_pos].trim().to_string();
                        extern_vars.push(var_name);
                    }
                }
                continue;
            }

            // Проверяем использование переменных от extern_fn
            if !extern_vars.is_empty() && Self::uses_any_var(trimmed, &extern_vars) {
                continue;
            }

            result.push_str(line);
            result.push('\n');
        }

        result
    }

    fn uses_any_var(line: &str, vars: &[String]) -> bool {
        for var in vars {
            // Ищем все вхождения переменной
            let var_bytes = var.as_bytes();
            let line_bytes = line.as_bytes();
            let var_len = var_bytes.len();

            let mut pos = 0;
            while pos + var_len <= line_bytes.len() {
                if let Some(found) = line[pos..].find(var.as_str()) {
                    let abs_pos = pos + found;
                    let before_pos = abs_pos;
                    let after_pos = abs_pos + var_len;

                    // Символ перед — не буква/цифра/подчёркивание
                    let before_ok = before_pos == 0 || {
                        let ch = line_bytes[before_pos - 1];
                        !ch.is_ascii_alphanumeric() && ch != b'_'
                    };

                    // Символ после — не буква/цифра/подчёркивание
                    let after_ok = after_pos >= line_bytes.len() || {
                        let ch = line_bytes[after_pos];
                        !ch.is_ascii_alphanumeric() && ch != b'_'
                    };

                    if before_ok && after_ok {
                        return true;
                    }

                    pos = abs_pos + 1;
                } else {
                    break;
                }
            }
        }
        false
    }
	fn compile_project(&mut self) {
		self.project_compiler_output.clear();
		self.project_run_output.clear();
		self.project_output.clear(); // ← ДОБАВЛЕНО: очищаем ВСЕ выводы проекта
		self.project_error_lines.clear();
		let t = self.texts();

		let full_source = self.collect_project_source();

		match compile_source(&full_source) {
			Ok((program, bytes)) => {
				let bytes_len = bytes.len();
				self.project_compiled_bytes = Some(bytes);
				self.project_compiler_output = format!(
					"{}\n{}: {}\n{}: {}\n{}: {}",
					t.compilation_success_detail,
					t.compilation_instructions,
					program.instructions.len(),
					t.compilation_functions_count,
					program.functions.len(),
					t.compilation_bytecode_size,
					bytes_len
				);
				self.status_msg = t.compilation_success.to_string();
				self.project_error_lines.clear();
				self.error_lines.clear(); // ← ДОБАВЛЕНО
			}
			Err(e) => {
				self.project_compiled_bytes = None;
				self.project_compiler_output = format!("{}\n{}", t.compilation_error_detail, e);
				self.status_msg = t.compilation_error.to_string();
				let errs = Self::parse_error_lines(&full_source, &e);
				self.error_lines = errs.clone();
				self.project_error_lines = errs;
			}
		}
	}

	fn run_project(&mut self) {
		self.project_run_output.clear();
		self.project_error_lines.clear(); // ← ДОБАВЛЕНО
		let t = self.texts();

		let full_source = self.collect_project_source();

		let has_extern = self
			.project_files
			.iter()
			.any(|f| f.content.contains("extern_fn "));
		let has_use_crate = self
			.project_files
			.iter()
			.any(|f| f.content.contains("use_crate("));
		let has_rust_block = self
			.project_files
			.iter()
			.any(|f| f.content.contains("rust {") || f.content.contains("rust{"));

		match compile_source(&full_source) {
			Ok((program, _bytes)) => match run_program(program) {
				Ok(output) => {
					let mut final_output = String::new();

					if has_extern || has_use_crate || has_rust_block {
						final_output.push_str("⚠️ SOME FUNCTIONS MISSED\n");
						final_output
							.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
						if has_use_crate {
							final_output.push_str("• use_crate(...) — needs to build .exe\n");
						}
						if has_rust_block {
							final_output.push_str("• rust { } blocks — needs to build .exe\n");
						}
						if has_extern {
							final_output.push_str("• extern_fn needs to build .exe\n");
						}
						final_output
							.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
						final_output.push_str("For full build press \"🚀 Build .exe\"\n\n");
					}

					if output.is_empty() {
						final_output.push_str(t.no_output);
					} else {
						final_output.push_str(&output.join("\n"));
					}
					self.project_run_output = final_output;
					self.status_msg = t.execution_done.to_string();
					self.error_lines.clear();
					self.project_error_lines.clear(); // ← ДОБАВЛЕНО
				}
				Err(e) => {
					self.project_run_output = format!("{}\n{}", t.execution_error_detail, e);
					self.status_msg = t.execution_error.to_string();
					let errs = Self::parse_error_lines(&full_source, &e); // ← ИСПРАВЛЕНО: full_source вместо self.source_code
					self.error_lines = errs.clone();
					self.project_error_lines = errs;
				}
			},
			Err(e) => {
				self.project_run_output = format!("{}\n{}", t.compilation_error_detail, e);
				self.status_msg = t.compilation_error.to_string();
				let errs = Self::parse_error_lines(&full_source, &e); // ← ИСПРАВЛЕНО
				self.error_lines = errs.clone();
				self.project_error_lines = errs;
			}
		}
	}

    fn save_project_sources(&mut self) {
        let t = self.texts();
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            for file in &self.project_files {
                let path = dir.join(&file.name);
                if let Err(e) = fs::write(&path, &file.content) {
                    self.status_msg = format!("{}: {}", t.status_save_error, e);
					self.error_lines = Self::parse_error_lines(&self.source_code, &e.to_string());
                    return;
                }
            }
            self.status_msg = format!("{}: {}", t.files_saved, dir.display());
        }
    }
	fn save_undo_snapshot(&mut self) {
        if self.source_code != self.last_saved_code {
            self.undo_stack.push(self.last_saved_code.clone());
            self.last_saved_code = self.source_code.clone();
            self.redo_stack.clear();
            // Ограничиваем размер стека
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
        }
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.source_code.clone());
            self.source_code = prev.clone();
            self.last_saved_code = prev;
            self.status_msg = self.texts().undo.to_string();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.source_code.clone());
            self.source_code = next.clone();
            self.last_saved_code = next;
            self.status_msg = self.texts().redo.to_string();
        }
    }
	fn save_project_files(&mut self) {
		let project_dir = std::path::PathBuf::from(&self.project_path);
		let _ = std::fs::create_dir_all(&project_dir);

		for file in &self.project_files {
			let path = project_dir.join(&file.name);
			let _ = std::fs::write(&path, &file.content);
		}

		// Обновляем кеш — теперь всё "сохранено"
		self.last_file_contents = self.project_files.iter()
			.map(|f| f.content.clone())
			.collect();
	}
	fn render_guide_window(&mut self, ctx: &egui::Context) {
		let mut show = self.show_guide;
		let lang = self.language;

		let title = match lang {
			Language::Ru => "📚 Полный гайд по Rust--",
			Language::En => "📚 Full Rust-- Guide",
		};

		egui::Window::new(title)
			.open(&mut show)
			.resizable(true)
			.default_width(900.0)
			.default_height(700.0)
			.vscroll(true)
			.show(ctx, |ui| {
				Self::render_guide_content_static(ui, lang);
			});

		self.show_guide = show;
	}
	
	fn render_settings_window(&mut self, ctx: &egui::Context) {
		let mut show = self.show_settings;
		let t = self.texts();
		let title = t.settings_title;

		egui::Window::new(title)
			.open(&mut show)
			.resizable(false)
			.default_width(500.0)
			.show(ctx, |ui| {
				ui.add_space(10.0);

				// Язык
				ui.group(|ui| {
					ui.label(
						egui::RichText::new(t.language)
							.size(16.0)
							.strong(),
					);
					ui.add_space(5.0);
					ui.horizontal(|ui| {
						if ui.selectable_label(self.language == Language::Ru, "🇷🇺 Русский").clicked() {
							self.language = Language::Ru;
							self.save_settings(); // ДОБАВИТЬ
						}
						ui.add_space(10.0);
						if ui.selectable_label(self.language == Language::En, "🇬🇧 English").clicked() {
							self.language = Language::En;
							self.save_settings(); // ДОБАВИТЬ
						}
					});
				});

				ui.add_space(10.0);

				// Тема
				ui.group(|ui| {
					ui.label(
						egui::RichText::new(t.theme_label)
							.size(16.0)
							.strong(),
					);
					ui.add_space(5.0);
					ui.horizontal(|ui| {
						if ui.selectable_label(self.theme_mode == ThemeMode::Dark, t.dark_mode).clicked() {
							self.theme_mode = ThemeMode::Dark;
							self.save_settings(); // ДОБАВИТЬ
						}
						ui.add_space(10.0);
						if ui.selectable_label(self.theme_mode == ThemeMode::Light, t.light_mode).clicked() {
							self.theme_mode = ThemeMode::Light;
							self.save_settings(); // ДОБАВИТЬ
						}
					});
				});

				ui.add_space(10.0);

				// Целевой код компиляции
				ui.group(|ui| {
					ui.label(
						egui::RichText::new(t.compile_target)
							.size(16.0)
							.strong(),
					);
					ui.add_space(5.0);
					
					ui.horizontal(|ui| {
						if ui.selectable_label(self.compile_target == CompileTarget::Rust, t.compile_to_rust).clicked() {
							self.compile_target = CompileTarget::Rust;
							self.save_settings(); // ДОБАВИТЬ
						}
						ui.add_space(10.0);
						if ui.selectable_label(self.compile_target == CompileTarget::Bytecode, t.compile_to_bytecode).clicked() {
							self.compile_target = CompileTarget::Bytecode;
							self.save_settings(); // ДОБАВИТЬ
						}
					});
				});

				ui.add_space(10.0);

				// Сжатие
				ui.group(|ui| {
					ui.label(
						egui::RichText::new(t.compression_level)
							.size(16.0)
							.strong(),
					);
					ui.add_space(5.0);
					
					if ui.selectable_label(self.compression_level == CompressionLevel::None, t.compression_none).clicked() {
						self.compression_level = CompressionLevel::None;
						self.save_settings(); // ДОБАВИТЬ
					}
					if ui.selectable_label(self.compression_level == CompressionLevel::Fast, t.compression_fast).clicked() {
						self.compression_level = CompressionLevel::Fast;
						self.save_settings(); // ДОБАВИТЬ
					}
					if ui.selectable_label(self.compression_level == CompressionLevel::Balanced, t.compression_balanced).clicked() {
						self.compression_level = CompressionLevel::Balanced;
						self.save_settings(); // ДОБАВИТЬ
					}
					if ui.selectable_label(self.compression_level == CompressionLevel::Max, t.compression_max).clicked() {
						self.compression_level = CompressionLevel::Max;
						self.save_settings(); // ДОБАВИТЬ
					}
					
					ui.add_space(5.0);
					ui.label(
						egui::RichText::new(t.compression_hint)
							.size(11.0)
							.italics()
							.color(egui::Color32::GRAY),
					);
				});
				ui.add_space(10.0);
				
				ui.group(|ui| {
					ui.label(
						egui::RichText::new(t.project_path_label)
							.size(16.0)
							.strong(),
					);
					ui.add_space(5.0);
					
					ui.label(
						egui::RichText::new(t.project_path_hint)
							.size(11.0)
							.italics()
							.color(egui::Color32::GRAY),
					);
					ui.add_space(5.0);

					// Показываем текущий путь (редактируемый)
					ui.horizontal(|ui| {
						let response = ui.add(
							egui::TextEdit::singleline(&mut self.project_path)
								.desired_width(350.0)
								.font(egui::FontId::monospace(12.0))
						);
						
						// Если пользователь вручную изменил путь и нажал Enter
						if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
							let new_path = std::path::PathBuf::from(&self.project_path);
							let _ = std::fs::create_dir_all(&new_path);
							self.save_settings();
							self.status_msg = t.project_path_changed.to_string();
						}
					});
					
					ui.add_space(5.0);
					
					ui.horizontal(|ui| {
						// Кнопка "Обзор..."
						if ui.button(t.project_path_browse).clicked() {
							if let Some(dir) = rfd::FileDialog::new()
								.set_directory(&self.project_path)
								.pick_folder()
							{
								self.project_path = dir.to_string_lossy().to_string();
								let _ = std::fs::create_dir_all(&dir);
								self.save_settings();
								self.status_msg = format!("{}: {}", t.project_path_changed, self.project_path);
							}
						}
						
						ui.add_space(10.0);
						
						// Кнопка "Перечитать файлы"
						if ui.button(t.project_path_reload).clicked() {
							self.reload_project_files();
						}
					});
					
					ui.add_space(5.0);
					
					// Показываем что сейчас в папке
					let project_dir = std::path::PathBuf::from(&self.project_path);
					if project_dir.exists() {
						let mut file_list = Vec::new();
						if let Ok(entries) = std::fs::read_dir(&project_dir) {
							for entry in entries.flatten() {
								let name = entry.file_name().to_string_lossy().to_string();
								if name.ends_with(".rsmm") {
									file_list.push(name);
								}
							}
						}
						
						if file_list.is_empty() {
							ui.label(
								egui::RichText::new("⚠ No .rsmm files in this folder")
									.color(egui::Color32::from_rgb(255, 200, 100))
									.size(11.0),
							);
						} else {
							file_list.sort();
							let files_str = file_list.join(", ");
							ui.label(
								egui::RichText::new(format!("📄 Files: {}", files_str))
									.color(egui::Color32::from_rgb(150, 200, 150))
									.size(11.0),
							);
						}
					} else {
						ui.label(
							egui::RichText::new("❌ Folder does not exist (will be created)")
								.color(egui::Color32::from_rgb(255, 100, 100))
								.size(11.0),
						);
					}
				});

				ui.add_space(15.0);

				// Информация
				ui.group(|ui| {
					ui.label(
						egui::RichText::new("Rust-- IDE")
							.size(14.0)
							.strong()
							.color(egui::Color32::from_rgb(100, 180, 255)),
					);
					ui.label("v1.0");
					ui.label(t.settings_description);
					ui.label(
						egui::RichText::new("easter steak")
							.size(1.0)
							.color(egui::Color32::from_rgb(100, 180, 255)),
					);	
				});
			});
		if !show && self.show_settings {
			self.save_settings();
		}
		self.show_settings = show;
	}

	fn render_guide_content_static(ui: &mut egui::Ui, lang: Language) {
		let heading = |ui: &mut egui::Ui, text: &str| {
			ui.add_space(10.0);
			ui.label(egui::RichText::new(text).size(22.0).strong().color(egui::Color32::from_rgb(100, 200, 255)));
			ui.add_space(5.0);
		};

		let subheading = |ui: &mut egui::Ui, text: &str| {
			ui.add_space(8.0);
			ui.label(egui::RichText::new(text).size(16.0).strong().color(egui::Color32::from_rgb(255, 200, 100)));
			ui.add_space(3.0);
		};

		let code_block = |ui: &mut egui::Ui, code: &str| {
			egui::Frame::none()
				.fill(egui::Color32::from_rgb(30, 30, 40))
				.rounding(5.0)
				.inner_margin(10.0)
				.show(ui, |ui| {
					ui.label(egui::RichText::new(code).monospace().color(egui::Color32::from_rgb(200, 200, 200)));
				});
		};

		match lang {
			Language::Ru => {
				// ========== ЗАГОЛОВОК ==========
				ui.vertical_centered(|ui| {
					ui.add_space(10.0);
					ui.label(egui::RichText::new("🦀 RUST-- v1.0").size(32.0).strong().color(egui::Color32::from_rgb(255, 150, 50)));
					ui.label(egui::RichText::new("Максимально упрощённый Rust").size(16.0).italics());
					ui.add_space(10.0);
				});

				ui.separator();

				heading(ui, "📑 Содержание");
				ui.label("1. Основы языка");
				ui.label("2. Типы данных");
				ui.label("3. Переменные");
				ui.label("4. Операторы");
				ui.label("5. Условия (if/else)");
				ui.label("6. Циклы (loop)");
				ui.label("7. Функции");
				ui.label("8. Мультифайловые проекты");
				ui.label("9. Подключение Rust-библиотек");
				ui.label("10. Внешние Rust-функции");
				ui.label("11. Встроенные функции");
				ui.label("12. Примеры программ");

				ui.separator();

				// ========== 1. ОСНОВЫ ==========
				heading(ui, "1. 🎯 Основы языка");
				ui.label("Rust-- — это упрощённый язык, который транспилируется в настоящий Rust и компилируется в нативный .exe через cargo.");
				ui.add_space(5.0);
				ui.label("• Файлы исходников: .rsmm");
				ui.label("• Скомпилированный байткод: .rsm");
				ui.label("• Финальный результат: .exe (нативный!)");

				subheading(ui, "Простейшая программа:");
				code_block(ui, r#"print("Привет, Rust--!");"#);

				ui.separator();

				// ========== 2. ТИПЫ ДАННЫХ ==========
				heading(ui, "2. 📦 Типы данных");

				subheading(ui, "Целые числа (int):");
				code_block(ui, "let x = 42;\nlet negative = -17;\nlet zero = 0;");

				subheading(ui, "Строки (str):");
				code_block(ui, "let name = \"Привет, мир!\";\nlet escaped = \"Строка с \\\"кавычками\\\" и \\n переносом\";");

				subheading(ui, "Логические (bool):");
				code_block(ui, "let flag = true;\nlet disabled = false;");

				ui.separator();

				// ========== 3. ПЕРЕМЕННЫЕ ==========
				heading(ui, "3. 📝 Переменные");

				subheading(ui, "Объявление:");
				code_block(ui, "let x = 10;          // целое число\nlet name = \"Rust\";   // строка\nlet flag = true;     // логическое");

				subheading(ui, "Переприсваивание:");
				code_block(ui, "let x = 10;\nx = 20;              // теперь x = 20\nx = x + 5;           // теперь x = 25");

				ui.separator();

				// ========== 4. ОПЕРАТОРЫ ==========
				heading(ui, "4. ⚙️ Операторы");

				subheading(ui, "Арифметические:");
				code_block(ui, "let a = 10 + 5;      // 15  (сложение)\nlet b = 10 - 5;      // 5   (вычитание)\nlet c = 10 * 5;      // 50  (умножение)\nlet d = 10 / 3;      // 3   (целочисленное деление)\nlet e = 10 % 3;      // 1   (остаток от деления)\nlet f = -42;         // -42 (отрицание)");

				subheading(ui, "Сравнения:");
				code_block(ui, "let eq = 10 == 10;   // true  (равно)\nlet ne = 10 != 5;    // true  (не равно)\nlet lt = 5 < 10;     // true  (меньше)\nlet gt = 10 > 5;     // true  (больше)\nlet le = 5 <= 5;     // true  (меньше или равно)\nlet ge = 10 >= 10;   // true  (больше или равно)");

				subheading(ui, "Логические:");
				code_block(ui, "let and = true && false;  // false (И)\nlet or = true || false;   // true  (ИЛИ)\nlet not = !true;          // false (НЕ)");

				subheading(ui, "Конкатенация строк:");
				code_block(ui, "let name = \"Мир\";\nlet age = 25;\nlet msg = \"Привет, \" + name + \"! Возраст: \" + age;\n// Результат: \"Привет, Мир! Возраст: 25\"");

				ui.separator();

				// ========== 5. УСЛОВИЯ ==========
				heading(ui, "5. 🔀 Условия (if/else)");

				subheading(ui, "Простое условие:");
				code_block(ui, "if x > 10 {\n    print(\"x больше 10\");\n}");

				subheading(ui, "If-else:");
				code_block(ui, "if x > 10 {\n    print(\"больше\");\n} else {\n    print(\"меньше или равно\");\n}");

				subheading(ui, "Вложенные условия:");
				code_block(ui, "if score >= 90 {\n    print(\"Отлично!\");\n} else {\n    if score >= 70 {\n        print(\"Хорошо\");\n    } else {\n        print(\"Нужно подтянуть\");\n    }\n}");

				subheading(ui, "Сложные условия:");
				code_block(ui, "if age >= 18 && has_ticket {\n    print(\"Добро пожаловать!\");\n}\n\nif is_admin || is_moderator {\n    print(\"Доступ разрешён\");\n}");

				ui.separator();

				// ========== 6. ЦИКЛЫ ==========
				heading(ui, "6. 🔄 Циклы (loop)");

				subheading(ui, "Бесконечный цикл с break:");
				code_block(ui, "let i = 0;\nloop {\n    if i >= 10 {\n        break;\n    }\n    print(i);\n    i = i + 1;\n}");

				subheading(ui, "Вложенные циклы:");
				code_block(ui, "let row = 1;\nloop {\n    if row > 3 {\n        break;\n    }\n    let col = 1;\n    loop {\n        if col > 3 {\n            break;\n        }\n        print(row * col);\n        col = col + 1;\n    }\n    row = row + 1;\n}");

				subheading(ui, "Сумма чисел:");
				code_block(ui, "let sum = 0;\nlet i = 1;\nloop {\n    if i > 100 {\n        break;\n    }\n    sum = sum + i;\n    i = i + 1;\n}\nprint(\"Сумма 1..100 = \" + sum);  // 5050");

				ui.separator();

				// ========== 7. ФУНКЦИИ ==========
				heading(ui, "7. 🔧 Функции");

				subheading(ui, "Определение функции:");
				code_block(ui, "fn greet(name) {\n    print(\"Привет, \" + name + \"!\");\n}\n\ngreet(\"Rust\");  // Привет, Rust!");

				subheading(ui, "Функция с несколькими параметрами:");
				code_block(ui, "fn add(a, b) {\n    let result = a + b;\n    print(a + \" + \" + b + \" = \" + result);\n}\n\nadd(10, 20);  // 10 + 20 = 30");

				subheading(ui, "Ранний выход (return):");
				code_block(ui, "fn check(n) {\n    if n < 0 {\n        print(\"Отрицательное!\");\n        return;\n    }\n    print(\"Положительное: \" + n);\n}\n\ncheck(-5);  // Отрицательное!\ncheck(10);  // Положительное: 10");

				subheading(ui, "Факториал:");
				code_block(ui, "fn factorial(n) {\n    let result = 1;\n    let i = 1;\n    loop {\n        if i > n {\n            break;\n        }\n        result = result * i;\n        i = i + 1;\n    }\n    print(n + \"! = \" + result);\n}\n\nfactorial(10);  // 10! = 3628800");

				ui.separator();

				// ========== 8. МУЛЬТИФАЙЛОВЫЕ ПРОЕКТЫ ==========
				heading(ui, "8. 📁 Мультифайловые проекты");

				subheading(ui, "Структура проекта:");
				code_block(ui, "📁 Проект\n├── ⭐ main.rsmm      (главный файл)\n├── 📄 math.rsmm      (математические функции)\n└── 📄 utils.rsmm     (утилиты)");

				subheading(ui, "Импорт файла (main.rsmm):");
				code_block(ui, "import(\"math.rsmm\");\nimport(\"utils.rsmm\");\n\nadd(10, 20);\nseparator();");

				subheading(ui, "Файл библиотеки (math.rsmm):");
				code_block(ui, "fn add(a, b) {\n    print(a + \" + \" + b + \" = \" + (a + b));\n}\n\nfn multiply(a, b) {\n    print(a + \" * \" + b + \" = \" + (a * b));\n}");

				ui.separator();

				// ========== 9. ПОДКЛЮЧЕНИЕ БИБЛИОТЕК ==========
				heading(ui, "9. 📦 Подключение Rust-библиотек");

				subheading(ui, "Синтаксис use_crate:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\nuse_crate(\"serde\", \"1.0\", \"derive\");\nuse_crate(\"tokio\", \"1\", \"full\", \"rt-multi-thread\");");

				subheading(ui, "Блок чистого Rust:");
				code_block(ui, "rust {\n    use rand::Rng;\n    use std::collections::HashMap;\n}");

				subheading(ui, "Популярные библиотеки:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");      // Случайные числа\nuse_crate(\"chrono\", \"0.4\");    // Дата и время\nuse_crate(\"colored\", \"2\");     // Цветной вывод\nuse_crate(\"ureq\", \"2\");        // HTTP запросы\nuse_crate(\"serde\", \"1\");       // Сериализация\nuse_crate(\"regex\", \"1\");       // Регулярные выражения");

				ui.separator();

				// ========== 10. ВНЕШНИЕ RUST-ФУНКЦИИ ==========
				heading(ui, "10. 🦀 Внешние Rust-функции (extern_fn)");

				ui.label("extern_fn позволяет писать функции на чистом Rust внутри Rust-- кода.");
				ui.add_space(5.0);

				subheading(ui, "Базовый синтаксис:");
				code_block(ui, "extern_fn имя(параметр1, параметр2) {\n    // Чистый Rust код\n    // Должен вернуть RsmmVal\n}");

				subheading(ui, "Типы RsmmVal:");
				code_block(ui, "RsmmVal::Int(42)                    // Целое число\nRsmmVal::Str(\"текст\".to_string())  // Строка\nRsmmVal::Bool(true)                 // Логическое\nRsmmVal::None                       // Ничего");

				subheading(ui, "Методы RsmmVal:");
				code_block(ui, "param.as_int()     // Получить i64\nparam.as_str()     // Получить String\nparam.as_bool()    // Получить bool");

				subheading(ui, "Пример: случайное число:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\n\nrust {\n    use rand::Rng;\n}\n\nextern_fn random(min, max) {\n    let mut rng = rand::thread_rng();\n    let n = rng.gen_range(min.as_int()..=max.as_int());\n    RsmmVal::Int(n)\n}\n\nlet x = random(1, 100);\nprint(\"Случайное число: \" + x);");

				subheading(ui, "Пример: работа с файлами:");
				code_block(ui, "extern_fn write_file(filename, content) {\n    match std::fs::write(filename.as_str(), content.as_str()) {\n        Ok(_) => RsmmVal::Bool(true),\n        Err(_) => RsmmVal::Bool(false),\n    }\n}\n\nextern_fn read_file(filename) {\n    match std::fs::read_to_string(filename.as_str()) {\n        Ok(s) => RsmmVal::Str(s),\n        Err(e) => RsmmVal::Str(format!(\"Ошибка: {}\", e)),\n    }\n}\n\nwrite_file(\"test.txt\", \"Привет!\");\nprint(read_file(\"test.txt\"));");

				ui.separator();

				// ========== 11. ВСТРОЕННЫЕ ФУНКЦИИ ==========
				heading(ui, "11. 🔨 Встроенные функции");

				subheading(ui, "Вывод:");
				code_block(ui, "print(\"Любой текст\");\nprint(42);\nprint(true);\nprint(\"x = \" + x);");

				subheading(ui, "Полезные extern_fn:");
				code_block(ui, "// Пауза\nextern_fn sleep_ms(ms) {\n    std::thread::sleep(std::time::Duration::from_millis(ms.as_int() as u64));\n    RsmmVal::None\n}\n\n// Ввод с клавиатуры\nextern_fn input(prompt) {\n    use std::io::Write;\n    print!(\"{}\", prompt.as_str());\n    std::io::stdout().flush().unwrap();\n    let mut buf = String::new();\n    std::io::stdin().read_line(&mut buf).unwrap();\n    RsmmVal::Str(buf.trim().to_string())\n}\n\n// Строка в число\nextern_fn to_int(val) {\n    RsmmVal::Int(val.as_str().parse::<i64>().unwrap_or(0))\n}\n\n// Длина строки\nextern_fn str_len(s) {\n    RsmmVal::Int(s.as_str().len() as i64)\n}");

				ui.separator();

				// ========== 12. ПРИМЕРЫ ==========
				heading(ui, "12. 💡 Примеры программ");

				subheading(ui, "Hello World:");
				code_block(ui, "print(\"Привет, Rust--!\");");

				subheading(ui, "FizzBuzz:");
				code_block(ui, "let i = 1;\nloop {\n    if i > 30 { break; }\n    if i % 15 == 0 {\n        print(\"FizzBuzz\");\n    } else {\n        if i % 3 == 0 {\n            print(\"Fizz\");\n        } else {\n            if i % 5 == 0 {\n                print(\"Buzz\");\n            } else {\n                print(i);\n            }\n        }\n    }\n    i = i + 1;\n}");

				subheading(ui, "Фибоначчи:");
				code_block(ui, "fn fibonacci(n) {\n    let a = 0;\n    let b = 1;\n    let i = 0;\n    loop {\n        if i >= n { break; }\n        print(\"fib(\" + i + \") = \" + a);\n        let temp = b;\n        b = a + b;\n        a = temp;\n        i = i + 1;\n    }\n}\n\nfibonacci(15);");

				subheading(ui, "Угадай число:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\n\nrust { use rand::Rng; }\n\nextern_fn random(min, max) {\n    RsmmVal::Int(rand::thread_rng().gen_range(min.as_int()..=max.as_int()))\n}\n\nextern_fn input(prompt) {\n    use std::io::Write;\n    print!(\"{}\", prompt.as_str());\n    std::io::stdout().flush().unwrap();\n    let mut buf = String::new();\n    std::io::stdin().read_line(&mut buf).unwrap();\n    RsmmVal::Str(buf.trim().to_string())\n}\n\nextern_fn to_int(s) {\n    RsmmVal::Int(s.as_str().parse().unwrap_or(0))\n}\n\nlet secret = random(1, 100);\nprint(\"Я загадал число от 1 до 100!\");\nlet attempts = 0;\nloop {\n    let guess = to_int(input(\"Попытка: \"));\n    attempts = attempts + 1;\n    if guess == secret {\n        print(\"Угадал за \" + attempts + \" попыток!\");\n        break;\n    }\n    if guess < secret { print(\"Больше!\"); }\n    else { print(\"Меньше!\"); }\n}");

				ui.separator();

				heading(ui, "📋 Шпаргалка");
				code_block(ui, "╔══════════════════════════════════════════╗\n║           RUST-- ШПАРГАЛКА               ║\n╠══════════════════════════════════════════╣\n║ ТИПЫ:      int, str, bool                ║\n║ ЛИТЕРАЛЫ:  42, \"текст\", true, false      ║\n║ ПЕРЕМЕННЫЕ: let x = 10; x = x + 1;      ║\n║ ОПЕРАТОРЫ: + - * / % == != < > && || !   ║\n║ ВЫВОД:     print(\"текст\");               ║\n║ УСЛОВИЕ:   if x > 0 { } else { }        ║\n║ ЦИКЛ:      loop { break; }              ║\n║ ФУНКЦИЯ:   fn name(a, b) { }            ║\n║ ИМПОРТ:    import(\"file.rsmm\");          ║\n║ КРЕЙТ:     use_crate(\"name\", \"ver\");     ║\n║ RUST КОД:  rust { use std::fs; }        ║\n║ EXTERN:    extern_fn name(x) { ... }    ║\n╚══════════════════════════════════════════╝");

				ui.add_space(20.0);
				ui.vertical_centered(|ui| {
					ui.label("Создано с ❤️");
				});
				ui.add_space(20.0);
			}

			Language::En => {
				// ========== HEADER ==========
				ui.vertical_centered(|ui| {
					ui.add_space(10.0);
					ui.label(egui::RichText::new("🦀 RUST-- v1.0").size(32.0).strong().color(egui::Color32::from_rgb(255, 150, 50)));
					ui.label(egui::RichText::new("Maximally simplified Rust").size(16.0).italics());
					ui.add_space(10.0);
				});

				ui.separator();

				heading(ui, "📑 Table of Contents");
				ui.label("1. Language Basics");
				ui.label("2. Data Types");
				ui.label("3. Variables");
				ui.label("4. Operators");
				ui.label("5. Conditions (if/else)");
				ui.label("6. Loops (loop)");
				ui.label("7. Functions");
				ui.label("8. Multi-file Projects");
				ui.label("9. Rust Libraries");
				ui.label("10. External Rust Functions");
				ui.label("11. Built-in Functions");
				ui.label("12. Example Programs");

				ui.separator();

				// ========== 1. BASICS ==========
				heading(ui, "1. 🎯 Language Basics");
				ui.label("Rust-- is a simplified language that transpiles to real Rust and compiles to native .exe via cargo.");
				ui.add_space(5.0);
				ui.label("• Source files: .rsmm");
				ui.label("• Compiled bytecode: .rsm");
				ui.label("• Final result: .exe (native!)");

				subheading(ui, "Simplest program:");
				code_block(ui, "print(\"Hello, Rust--!\");");

				ui.separator();

				// ========== 2. DATA TYPES ==========
				heading(ui, "2. 📦 Data Types");

				subheading(ui, "Integers (int):");
				code_block(ui, "let x = 42;\nlet negative = -17;\nlet zero = 0;");

				subheading(ui, "Strings (str):");
				code_block(ui, "let name = \"Hello, world!\";\nlet escaped = \"String with \\\"quotes\\\" and \\n newline\";");

				subheading(ui, "Booleans (bool):");
				code_block(ui, "let flag = true;\nlet disabled = false;");

				ui.separator();

				// ========== 3. VARIABLES ==========
				heading(ui, "3. 📝 Variables");

				subheading(ui, "Declaration:");
				code_block(ui, "let x = 10;          // integer\nlet name = \"Rust\";   // string\nlet flag = true;     // boolean");

				subheading(ui, "Reassignment:");
				code_block(ui, "let x = 10;\nx = 20;              // now x = 20\nx = x + 5;           // now x = 25");

				ui.separator();

				// ========== 4. OPERATORS ==========
				heading(ui, "4. ⚙️ Operators");

				subheading(ui, "Arithmetic:");
				code_block(ui, "let a = 10 + 5;      // 15  (addition)\nlet b = 10 - 5;      // 5   (subtraction)\nlet c = 10 * 5;      // 50  (multiplication)\nlet d = 10 / 3;      // 3   (integer division)\nlet e = 10 % 3;      // 1   (remainder)\nlet f = -42;         // -42 (negation)");

				subheading(ui, "Comparison:");
				code_block(ui, "let eq = 10 == 10;   // true  (equal)\nlet ne = 10 != 5;    // true  (not equal)\nlet lt = 5 < 10;     // true  (less than)\nlet gt = 10 > 5;     // true  (greater than)\nlet le = 5 <= 5;     // true  (less or equal)\nlet ge = 10 >= 10;   // true  (greater or equal)");

				subheading(ui, "Logical:");
				code_block(ui, "let and = true && false;  // false (AND)\nlet or = true || false;   // true  (OR)\nlet not = !true;          // false (NOT)");

				subheading(ui, "String concatenation:");
				code_block(ui, "let name = \"World\";\nlet age = 25;\nlet msg = \"Hello, \" + name + \"! Age: \" + age;\n// Result: \"Hello, World! Age: 25\"");

				ui.separator();

				// ========== 5. CONDITIONS ==========
				heading(ui, "5. 🔀 Conditions (if/else)");

				subheading(ui, "Simple condition:");
				code_block(ui, "if x > 10 {\n    print(\"x is greater than 10\");\n}");

				subheading(ui, "If-else:");
				code_block(ui, "if x > 10 {\n    print(\"greater\");\n} else {\n    print(\"less or equal\");\n}");

				subheading(ui, "Nested conditions:");
				code_block(ui, "if score >= 90 {\n    print(\"Excellent!\");\n} else {\n    if score >= 70 {\n        print(\"Good\");\n    } else {\n        print(\"Needs improvement\");\n    }\n}");

				subheading(ui, "Complex conditions:");
				code_block(ui, "if age >= 18 && has_ticket {\n    print(\"Welcome!\");\n}\n\nif is_admin || is_moderator {\n    print(\"Access granted\");\n}");

				ui.separator();

				// ========== 6. LOOPS ==========
				heading(ui, "6. 🔄 Loops (loop)");

				subheading(ui, "Infinite loop with break:");
				code_block(ui, "let i = 0;\nloop {\n    if i >= 10 {\n        break;\n    }\n    print(i);\n    i = i + 1;\n}");

				subheading(ui, "Nested loops:");
				code_block(ui, "let row = 1;\nloop {\n    if row > 3 { break; }\n    let col = 1;\n    loop {\n        if col > 3 { break; }\n        print(row * col);\n        col = col + 1;\n    }\n    row = row + 1;\n}");

				subheading(ui, "Sum of numbers:");
				code_block(ui, "let sum = 0;\nlet i = 1;\nloop {\n    if i > 100 { break; }\n    sum = sum + i;\n    i = i + 1;\n}\nprint(\"Sum 1..100 = \" + sum);  // 5050");

				ui.separator();

				// ========== 7. FUNCTIONS ==========
				heading(ui, "7. 🔧 Functions");

				subheading(ui, "Function definition:");
				code_block(ui, "fn greet(name) {\n    print(\"Hello, \" + name + \"!\");\n}\n\ngreet(\"Rust\");  // Hello, Rust!");

				subheading(ui, "Multiple parameters:");
				code_block(ui, "fn add(a, b) {\n    let result = a + b;\n    print(a + \" + \" + b + \" = \" + result);\n}\n\nadd(10, 20);  // 10 + 20 = 30");

				subheading(ui, "Early return:");
				code_block(ui, "fn check(n) {\n    if n < 0 {\n        print(\"Negative!\");\n        return;\n    }\n    print(\"Positive: \" + n);\n}\n\ncheck(-5);  // Negative!\ncheck(10);  // Positive: 10");

				subheading(ui, "Factorial:");
				code_block(ui, "fn factorial(n) {\n    let result = 1;\n    let i = 1;\n    loop {\n        if i > n { break; }\n        result = result * i;\n        i = i + 1;\n    }\n    print(n + \"! = \" + result);\n}\n\nfactorial(10);  // 10! = 3628800");

				ui.separator();

				// ========== 8. MULTI-FILE PROJECTS ==========
				heading(ui, "8. 📁 Multi-file Projects");

				subheading(ui, "Project structure:");
				code_block(ui, "📁 Project\n├── ⭐ main.rsmm      (main file)\n├── 📄 math.rsmm      (math functions)\n└── 📄 utils.rsmm     (utilities)");

				subheading(ui, "Importing files (main.rsmm):");
				code_block(ui, "import(\"math.rsmm\");\nimport(\"utils.rsmm\");\n\nadd(10, 20);\nseparator();");

				subheading(ui, "Library file (math.rsmm):");
				code_block(ui, "fn add(a, b) {\n    print(a + \" + \" + b + \" = \" + (a + b));\n}\n\nfn multiply(a, b) {\n    print(a + \" * \" + b + \" = \" + (a * b));\n}");

				ui.separator();

				// ========== 9. RUST LIBRARIES ==========
				heading(ui, "9. 📦 Rust Libraries");

				subheading(ui, "use_crate syntax:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\nuse_crate(\"serde\", \"1.0\", \"derive\");\nuse_crate(\"tokio\", \"1\", \"full\", \"rt-multi-thread\");");

				subheading(ui, "Pure Rust block:");
				code_block(ui, "rust {\n    use rand::Rng;\n    use std::collections::HashMap;\n}");

				subheading(ui, "Popular libraries:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");      // Random numbers\nuse_crate(\"chrono\", \"0.4\");    // Date and time\nuse_crate(\"colored\", \"2\");     // Colored output\nuse_crate(\"ureq\", \"2\");        // HTTP requests\nuse_crate(\"serde\", \"1\");       // Serialization\nuse_crate(\"regex\", \"1\");       // Regular expressions");

				ui.separator();

				// ========== 10. EXTERNAL RUST FUNCTIONS ==========
				heading(ui, "10. 🦀 External Rust Functions (extern_fn)");

				ui.label("extern_fn lets you write pure Rust functions inside Rust-- code.");
				ui.add_space(5.0);

				subheading(ui, "Basic syntax:");
				code_block(ui, "extern_fn name(param1, param2) {\n    // Pure Rust code\n    // Must return RsmmVal\n}");

				subheading(ui, "RsmmVal types:");
				code_block(ui, "RsmmVal::Int(42)                    // Integer\nRsmmVal::Str(\"text\".to_string())   // String\nRsmmVal::Bool(true)                 // Boolean\nRsmmVal::None                       // Nothing");

				subheading(ui, "RsmmVal methods:");
				code_block(ui, "param.as_int()     // Get i64\nparam.as_str()     // Get String\nparam.as_bool()    // Get bool");

				subheading(ui, "Example: random number:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\n\nrust {\n    use rand::Rng;\n}\n\nextern_fn random(min, max) {\n    let mut rng = rand::thread_rng();\n    let n = rng.gen_range(min.as_int()..=max.as_int());\n    RsmmVal::Int(n)\n}\n\nlet x = random(1, 100);\nprint(\"Random number: \" + x);");

				subheading(ui, "Example: file operations:");
				code_block(ui, "extern_fn write_file(filename, content) {\n    match std::fs::write(filename.as_str(), content.as_str()) {\n        Ok(_) => RsmmVal::Bool(true),\n        Err(_) => RsmmVal::Bool(false),\n    }\n}\n\nextern_fn read_file(filename) {\n    match std::fs::read_to_string(filename.as_str()) {\n        Ok(s) => RsmmVal::Str(s),\n        Err(e) => RsmmVal::Str(format!(\"Error: {}\", e)),\n    }\n}\n\nwrite_file(\"test.txt\", \"Hello!\");\nprint(read_file(\"test.txt\"));");

				ui.separator();

				// ========== 11. BUILT-IN FUNCTIONS ==========
				heading(ui, "11. 🔨 Built-in Functions");

				subheading(ui, "Output:");
				code_block(ui, "print(\"Any text\");\nprint(42);\nprint(true);\nprint(\"x = \" + x);");

				subheading(ui, "Useful extern_fn snippets:");
				code_block(ui, "// Sleep\nextern_fn sleep_ms(ms) {\n    std::thread::sleep(std::time::Duration::from_millis(ms.as_int() as u64));\n    RsmmVal::None\n}\n\n// Keyboard input\nextern_fn input(prompt) {\n    use std::io::Write;\n    print!(\"{}\", prompt.as_str());\n    std::io::stdout().flush().unwrap();\n    let mut buf = String::new();\n    std::io::stdin().read_line(&mut buf).unwrap();\n    RsmmVal::Str(buf.trim().to_string())\n}\n\n// String to integer\nextern_fn to_int(val) {\n    RsmmVal::Int(val.as_str().parse::<i64>().unwrap_or(0))\n}\n\n// String length\nextern_fn str_len(s) {\n    RsmmVal::Int(s.as_str().len() as i64)\n}");

				ui.separator();

				// ========== 12. EXAMPLES ==========
				heading(ui, "12. 💡 Example Programs");

				subheading(ui, "Hello World:");
				code_block(ui, "print(\"Hello, Rust--!\");");

				subheading(ui, "FizzBuzz:");
				code_block(ui, "let i = 1;\nloop {\n    if i > 30 { break; }\n    if i % 15 == 0 {\n        print(\"FizzBuzz\");\n    } else {\n        if i % 3 == 0 {\n            print(\"Fizz\");\n        } else {\n            if i % 5 == 0 {\n                print(\"Buzz\");\n            } else {\n                print(i);\n            }\n        }\n    }\n    i = i + 1;\n}");

				subheading(ui, "Fibonacci:");
				code_block(ui, "fn fibonacci(n) {\n    let a = 0;\n    let b = 1;\n    let i = 0;\n    loop {\n        if i >= n { break; }\n        print(\"fib(\" + i + \") = \" + a);\n        let temp = b;\n        b = a + b;\n        a = temp;\n        i = i + 1;\n    }\n}\n\nfibonacci(15);");

				subheading(ui, "Guess the number:");
				code_block(ui, "use_crate(\"rand\", \"0.8\");\n\nrust { use rand::Rng; }\n\nextern_fn random(min, max) {\n    RsmmVal::Int(rand::thread_rng().gen_range(min.as_int()..=max.as_int()))\n}\n\nextern_fn input(prompt) {\n    use std::io::Write;\n    print!(\"{}\", prompt.as_str());\n    std::io::stdout().flush().unwrap();\n    let mut buf = String::new();\n    std::io::stdin().read_line(&mut buf).unwrap();\n    RsmmVal::Str(buf.trim().to_string())\n}\n\nextern_fn to_int(s) {\n    RsmmVal::Int(s.as_str().parse().unwrap_or(0))\n}\n\nlet secret = random(1, 100);\nprint(\"I picked a number from 1 to 100!\");\nlet attempts = 0;\nloop {\n    let guess = to_int(input(\"Your guess: \"));\n    attempts = attempts + 1;\n    if guess == secret {\n        print(\"Got it in \" + attempts + \" attempts!\");\n        break;\n    }\n    if guess < secret { print(\"Higher!\"); }\n    else { print(\"Lower!\"); }\n}");

				ui.separator();

				heading(ui, "📋 Cheat Sheet");
				code_block(ui, "╔══════════════════════════════════════════╗\n║          RUST-- CHEAT SHEET              ║\n╠══════════════════════════════════════════╣\n║ TYPES:     int, str, bool                ║\n║ LITERALS:  42, \"text\", true, false       ║\n║ VARIABLES: let x = 10; x = x + 1;       ║\n║ OPERATORS: + - * / % == != < > && || !   ║\n║ OUTPUT:    print(\"text\");                ║\n║ CONDITION: if x > 0 { } else { }        ║\n║ LOOP:      loop { break; }              ║\n║ FUNCTION:  fn name(a, b) { }            ║\n║ IMPORT:    import(\"file.rsmm\");          ║\n║ CRATE:     use_crate(\"name\", \"ver\");     ║\n║ RUST CODE: rust { use std::fs; }        ║\n║ EXTERN:    extern_fn name(x) { ... }    ║\n╚══════════════════════════════════════════╝");

				ui.add_space(20.0);
				ui.vertical_centered(|ui| {
					ui.label("Created with ❤️");
				});
				ui.add_space(20.0);
			}
		}
	}
	fn render_project_tab(&mut self, ui: &mut egui::Ui) {
		let t = self.texts();
		let run_btn = t.project_run;
		let compile_btn = t.project_compile;
		let save_src_btn = t.project_save_source;
		let build_btn = t.project_build_btn;
		let inst_btn = t.project_installer_btn;
		let show_btn = t.project_show_rust_btn;
		let proj_lbl = t.project_label_name;
		let save_files_btn = t.save_files;
		let files_saved_msg = t.files_saved;
		let files_lbl = t.project_files;
		let deps_lbl = t.dependencies;
		let add_crate_btn = t.add_crate;
		let dep_name_lbl = t.dep_name;
		let dep_ver_lbl = t.dep_version;
		let dep_feat_lbl = t.dep_features;
		let result_lbl = t.result;
		let new_file_cmt = t.new_file_comment;
		let compile_out_lbl = t.project_compile_output;
		let run_out_lbl = t.project_compiler_output;

		// Тулбар
		ui.horizontal(|ui| {
			if ui
				.button(egui::RichText::new(run_btn).color(egui::Color32::GREEN))
				.clicked()
			{
				self.save_project_files(); // ← сохраняем перед запуском
				self.compile_project();
				self.run_project();
			}

			if ui.button(compile_btn).clicked() {
				self.save_project_files(); // ← сохраняем перед компиляцией
				self.compile_project();
			}

			ui.separator();

			// ← НОВАЯ КНОПКА: Сохранить всё
			if ui.button(
				egui::RichText::new("💾 Save")
					.color(egui::Color32::from_rgb(100, 200, 255))
			).clicked() {
				self.save_project_files();
				self.status_msg = files_saved_msg.to_string();
			}

			ui.separator();

			if ui
				.button(egui::RichText::new(build_btn).color(egui::Color32::from_rgb(255, 200, 50)))
				.clicked()
			{
				self.save_project_files(); // ← сохраняем перед сборкой
				self.build_project();
			}

			ui.menu_button(
				egui::RichText::new("⚡ LLVM")
					.color(egui::Color32::from_rgb(255, 150, 50)),
				|ui| {
					ui.label(egui::RichText::new("🔧 LLVM компиляция").size(13.0).strong());
					ui.separator();
					
					if ui.button("📄 Сохранить .ll (LLVM IR)").clicked() {
						// ... генерация IR
						ui.close_menu();
					}
					
					if ui.button("🔗 LLVM → .exe").clicked() {
						if let Some(path) = rfd::FileDialog::new()
							.add_filter("Executable", &["exe"])
							.set_directory(&self.project_path)
							.set_file_name(&format!("{}_llvm.exe", self.project_name))
							.save_file()
						{
							self.save_project_files();
							
							let mut project = lang::Project::new(&self.project_name);
							for file in &self.project_files {
								project.add_file(&file.name, &file.content, file.is_main);
							}
							for dep in &self.project_deps {
								if dep.features.is_empty() {
									project.add_dep(&dep.name, &dep.version);
								} else {
									let feats: Vec<&str> = dep.features.split(',').map(|s| s.trim()).collect();
									project.add_dep_with_features(&dep.name, &dep.version, feats);
								}
							}
							
							match lang::build_project_llvm(&project, &path) {
								Ok(info) => {
									self.project_output = info;
									self.status_msg = "✅ LLVM EXE".into();
									self.project_error_lines.clear();
								}
								Err(e) => {
									self.project_output = format!("❌ {}", e);
									self.status_msg = "Ошибка".into();
								}
							}
						}
						ui.close_menu();
					}
					
					ui.separator();
					ui.collapsing("❓ Требования", |ui| {
						ui.label("LLVM/Clang:");
						if ui.small_button("📥 releases.llvm.org").clicked() {
							let _ = open::that("https://releases.llvm.org/");
						}
						ui.label("Rust (для runtime):");
						if ui.small_button("📥 rustup.rs").clicked() {
							let _ = open::that("https://rustup.rs/");
						}
					});
				}
			);

			if ui.button(inst_btn).clicked() {
				self.save_project_files(); // ← сохраняем перед сборкой установщика
				self.build_project_installer();
			}

			if ui.button(show_btn).clicked() {
				self.show_project_rust();
			}

			ui.separator();

			if ui.button(save_src_btn).clicked() {
				self.save_project_sources();
			}

			ui.separator();

			ui.label(proj_lbl);
			ui.text_edit_singleline(&mut self.project_name);
		});

		ui.separator();

		ui.columns(3, |cols| {
			// Колонка 1: Файлы проекта + зависимости
			cols[0].label(
				egui::RichText::new(files_lbl)
					.strong()
					.color(egui::Color32::from_rgb(255, 200, 100)),
			);

			// Список файлов
			let mut clicked_idx = None;
			let mut delete_idx = None;

			for (i, file) in self.project_files.iter().enumerate() {
				cols[0].horizontal(|ui| {
					let label = if file.is_main {
						format!("⭐ {}", file.name)
					} else {
						format!("📄 {}", file.name)
					};
					let color = if i == self.active_file_idx {
						egui::Color32::from_rgb(100, 200, 255)
					} else {
						egui::Color32::from_rgb(200, 200, 200)
					};
					if ui
						.selectable_label(
							i == self.active_file_idx,
							egui::RichText::new(&label).color(color),
						)
						.clicked()
					{
						clicked_idx = Some(i);
					}
					if !file.is_main && ui.small_button("🗑").clicked() {
						delete_idx = Some(i);
					}
				});
			}

			if let Some(idx) = clicked_idx {
				self.active_file_idx = idx;
			}
			// ← ИЗМЕНЕНО: удаление через новый метод
			if let Some(idx) = delete_idx {
				self.delete_project_file(idx);
			}

			// Добавить файл — ← ИЗМЕНЕНО: через новый метод
			cols[0].separator();
			cols[0].horizontal(|ui| {
				ui.text_edit_singleline(&mut self.new_file_name);
				if ui.button("➕").clicked() && !self.new_file_name.is_empty() {
					let name = if self.new_file_name.ends_with(".rsmm") {
						self.new_file_name.clone()
					} else {
						format!("{}.rsmm", self.new_file_name)
					};
					self.create_new_file(name, new_file_cmt.to_string(), false);
					self.new_file_name.clear();
				}
			});

			// Зависимости
			cols[0].separator();
			cols[0].label(
				egui::RichText::new(deps_lbl)
					.strong()
					.color(egui::Color32::from_rgb(200, 255, 200)),
			);

			let mut dep_delete = None;
			for (i, dep) in self.project_deps.iter().enumerate() {
				cols[0].horizontal(|ui| {
					let feat_str = if dep.features.is_empty() {
						String::new()
					} else {
						format!(" [{}]", dep.features)
					};
					ui.label(format!("📦 {} v{}{}", dep.name, dep.version, feat_str));
					if ui.small_button("🗑").clicked() {
						dep_delete = Some(i);
					}
				});
			}
			if let Some(idx) = dep_delete {
				self.project_deps.remove(idx);
			}

			cols[0].separator();
			cols[0].horizontal(|ui| {
				ui.label(dep_name_lbl);
				ui.add(egui::TextEdit::singleline(&mut self.new_dep_name).desired_width(80.0));
			});
			cols[0].horizontal(|ui| {
				ui.label(dep_ver_lbl);
				ui.add(egui::TextEdit::singleline(&mut self.new_dep_version).desired_width(80.0));
			});
			cols[0].horizontal(|ui| {
				ui.label(dep_feat_lbl);
				ui.add(egui::TextEdit::singleline(&mut self.new_dep_features).desired_width(80.0));
			});
			if cols[0].button(add_crate_btn).clicked() && !self.new_dep_name.is_empty() {
				let ver = if self.new_dep_version.is_empty() {
					"*".to_string()
				} else {
					self.new_dep_version.clone()
				};
				self.project_deps.push(DepUI {
					name: self.new_dep_name.clone(),
					version: ver,
					features: self.new_dep_features.clone(),
				});
				self.new_dep_name.clear();
				self.new_dep_version.clear();
				self.new_dep_features.clear();
			}

			// Колонка 2: Редактор активного файла
			if let Some(file) = self.project_files.get_mut(self.active_file_idx) {
				cols[1].horizontal(|ui| {
					ui.label(
						egui::RichText::new(format!("📝 {}", file.name))
							.strong()
							.color(egui::Color32::from_rgb(255, 200, 100)),
					);
					// ← НОВОЕ: индикатор несохранённых изменений
					let is_modified = if self.active_file_idx < self.last_file_contents.len() {
						file.content != self.last_file_contents[self.active_file_idx]
					} else {
						true
					};
					if is_modified {
						ui.label(
							egui::RichText::new("● modified")
								.size(11.0)
								.color(egui::Color32::from_rgb(255, 200, 50)),
						);
					} else {
						ui.label(
							egui::RichText::new("✓ saved")
								.size(11.0)
								.color(egui::Color32::from_rgb(100, 200, 100)),
						);
					}
				});
				syntax_highlight::code_editor(&mut cols[1], &mut file.content, "project_editor", &self.project_error_lines);
			}

			// Колонка 3: Вывод
			cols[2].label(
				egui::RichText::new(result_lbl)
					.strong()
					.color(egui::Color32::from_rgb(200, 255, 200)),
			);

			egui::ScrollArea::vertical()
				.id_source("project_output_scroll")
				.show(&mut cols[2], |ui| {
					if !self.project_compiler_output.is_empty() {
						ui.label(
							egui::RichText::new(compile_out_lbl)
								.strong()
								.color(egui::Color32::from_rgb(200, 255, 200)),
						);
						let color = if self.project_compiler_output.contains('❌') {
							egui::Color32::from_rgb(255, 100, 100)
						} else {
							egui::Color32::from_rgb(100, 255, 100)
						};
						ui.colored_label(color, &self.project_compiler_output);
						ui.separator();
					}

					if !self.project_run_output.is_empty() {
						ui.label(
							egui::RichText::new(run_out_lbl)
								.strong()
								.color(egui::Color32::from_rgb(100, 200, 255)),
						);

						let color = if self.project_run_output.contains('❌') {
							egui::Color32::from_rgb(255, 100, 100)
						} else {
							egui::Color32::from_rgb(220, 220, 220)
						};

						for (i, line) in self.project_run_output.lines().enumerate() {
							ui.horizontal(|ui| {
								ui.label(
									egui::RichText::new(format!("{:>3}│", i + 1))
										.color(egui::Color32::from_rgb(100, 100, 100))
										.monospace(),
								);
								ui.label(
									egui::RichText::new(line).color(color).monospace(),
								);
							});
						}
					}

					if !self.project_output.is_empty() {
						ui.separator();
						let color = if self.project_output.contains('❌') {
							egui::Color32::from_rgb(255, 100, 100)
						} else {
							egui::Color32::from_rgb(200, 200, 200)
						};
						ui.colored_label(color, &self.project_output);
					}
				});
		});
	}
	
	fn build_project_installer(&mut self) {
		self.project_compiler_output.clear(); // ← ДОБАВЛЕНО
		self.project_error_lines.clear(); // ← ДОБАВЛЕНО

		let mut project = lang::Project::new(&self.project_name);

		for file in &self.project_files {
			project.add_file(&file.name, &file.content, file.is_main);
		}

		for dep in &self.project_deps {
			if dep.features.is_empty() {
				project.add_dep(&dep.name, &dep.version);
			} else {
				let feats: Vec<&str> = dep.features.split(',').map(|s| s.trim()).collect();
				project.add_dep_with_features(&dep.name, &dep.version, feats);
			}
		}

		if let Some(path) = rfd::FileDialog::new()
			.add_filter("Installer", &["exe", "msi"])
			.set_file_name(&format!("{}_setup.exe", self.project_name))
			.save_file()
		{
			match lang::build_project_msi(&project, &path) {
				Ok(info) => {
					self.project_compiler_output = info;
					self.status_msg = format!("✅ Installer: {}", path.display());
					self.error_lines.clear();
					self.project_error_lines.clear(); // ← ДОБАВЛЕНО
				}
				Err(e) => {
					self.project_compiler_output = format!("❌ {}", e);
					self.status_msg = "Error".into();
					let errs = Self::parse_error_lines(
						&self.collect_project_source(),
						&e,
					);
					self.error_lines = errs.clone();
					self.project_error_lines = errs;
				}
			}
		}
	}

	fn build_project(&mut self) {
		self.project_output.clear(); // ← ДОБАВЛЕНО
		self.project_error_lines.clear(); // ← ДОБАВЛЕНО

		let mut project = lang::Project::new(&self.project_name);

		for file in &self.project_files {
			project.add_file(&file.name, &file.content, file.is_main);
		}

		for dep in &self.project_deps {
			if dep.features.is_empty() {
				project.add_dep(&dep.name, &dep.version);
			} else {
				let feats: Vec<&str> = dep.features.split(',').map(|s| s.trim()).collect();
				project.add_dep_with_features(&dep.name, &dep.version, feats);
			}
		}

		if let Some(path) = rfd::FileDialog::new()
			.add_filter("Executable", &["exe"])
			.set_file_name(&format!("{}.exe", self.project_name))
			.save_file()
		{
			let profile = self.get_cargo_profile();
			match lang::build_project_exe_with_compression(&project, &path, profile) {
				Ok(info) => {
					self.project_output = info;
					self.status_msg = format!("✅ Builded: {}", path.display());
					self.error_lines.clear();
					self.project_error_lines.clear(); // ← ДОБАВЛЕНО
				}
				Err(e) => {
					self.project_output = format!("❌ {}", e);
					self.status_msg = "Err".into();
					let errs = Self::parse_error_lines(
						&self.collect_project_source(),
						&e,
					);
					self.error_lines = errs.clone();
					self.project_error_lines = errs;
				}
			}
		}
	}

	fn show_project_rust(&mut self) {
		self.project_compiler_output.clear(); // ← ДОБАВЛЕНО
		self.project_error_lines.clear(); // ← ДОБАВЛЕНО

		let mut project = lang::Project::new(&self.project_name);
		for file in &self.project_files {
			project.add_file(&file.name, &file.content, file.is_main);
		}
		for dep in &self.project_deps {
			if dep.features.is_empty() {
				project.add_dep(&dep.name, &dep.version);
			} else {
				let feats: Vec<&str> = dep.features.split(',').map(|s| s.trim()).collect();
				project.add_dep_with_features(&dep.name, &dep.version, feats);
			}
		}

		let mut transpiler = lang::ExtTranspiler::new();
		match transpiler.transpile_project(&project) {
			Ok(result) => {
				self.project_compiler_output = format!(
					"=== Cargo.toml ===\n{}\n\n=== main.rs ===\n{}",
					result.cargo_toml, result.rust_code
				);
				self.status_msg = "Rust code generated".into();
				self.error_lines.clear();
				self.project_error_lines.clear(); // ← ДОБАВЛЕНО
			}
			Err(e) => {
				self.project_compiler_output = format!("❌ {}", e);
				let errs = Self::parse_error_lines(
					&self.collect_project_source(),
					&e,
				);
				self.error_lines = errs.clone();
				self.project_error_lines = errs;
			}
		}
	}
	
	fn render_compiler_tab(&mut self, ui: &mut egui::Ui) {
		// Toolbar
		ui.horizontal(|ui| {
			if ui
				.button(egui::RichText::new(self.texts().run).color(egui::Color32::GREEN))
				.clicked()
			{
				self.compile();
				self.run();
			}
			if ui.button(self.texts().compile).clicked() {
				self.compile();
			}
			if ui.button(self.texts().save_rsm).clicked() {
				self.compile();
				self.save_compiled();
			}
			if ui
				.button(
					egui::RichText::new(self.texts().build_exe)
						.color(egui::Color32::from_rgb(255, 200, 50)),
				)
				.clicked()
			{
				if let Some(path) = rfd::FileDialog::new()
					.add_filter("Executable", &["exe"])
					.set_file_name("program.exe")
					.save_file()
				{
					let profile = self.get_cargo_profile();
					match lang::build_exe_with_compression(&self.source_code, &path, profile) {
						Ok(info) => {
							self.compiler_output = info;
							self.status_msg = format!("✅ EXE: {}", path.display());
							self.error_lines.clear();
						}
						Err(e) => {
							self.compiler_output = format!("❌ {}", e);
							self.status_msg = "Err".into();
							self.error_lines = Self::parse_error_lines(&self.source_code, &e);
						}
					}
				}
			}
			// После кнопки build_exe:
			ui.menu_button(
				egui::RichText::new("⚡ Native")
					.color(egui::Color32::from_rgb(255, 100, 100)),
				|ui| {
					ui.label(
						egui::RichText::new("🔧 Нативная компиляция")
							.size(13.0)
							.strong()
					);
					ui.separator();
					
					if ui.button("📄 Сохранить .asm").clicked() {
						if let Some(path) = rfd::FileDialog::new()
							.add_filter("Assembly", &["asm"])
							.set_file_name("program.asm")
							.save_file()
						{
							match lang::generate_asm(&self.source_code, &path) {
								Ok(info) => {
									self.compiler_output = info;
									self.status_msg = "✅ ASM".into();
									self.error_lines.clear();
								}
								Err(e) => {
									self.compiler_output = format!("❌ {}", e);
									self.status_msg = "Error".into();
									self.error_lines = Self::parse_error_lines(&self.source_code, &e);
								}
							}
						}
						ui.close_menu();
					}
					
					ui.separator();
					ui.label(egui::RichText::new("📦 Без библиотек:").size(11.0).color(egui::Color32::GRAY));
					
					if ui.button("🔧 NASM → .exe").clicked() {
						if let Some(path) = rfd::FileDialog::new()
							.add_filter("Executable", &["exe"])
							.set_file_name("program_nasm.exe")
							.save_file()
						{
							match lang::build_native_exe(&self.source_code, &path) {
								Ok(info) => {
									self.compiler_output = info;
									self.status_msg = "✅ NASM EXE".into();
									self.error_lines.clear();
								}
								Err(e) => {
									self.compiler_output = format!("❌ {}", e);
									self.status_msg = "Error".into();
									self.error_lines = Self::parse_error_lines(&self.source_code, &e);
								}
							}
						}
						ui.close_menu();
					}
					
					ui.separator();
					ui.label(egui::RichText::new("🦀 С библиотеками:").size(11.0).color(egui::Color32::GRAY));
					
					if ui.button("🔗 Гибрид → .exe").clicked() {
						if let Some(path) = rfd::FileDialog::new()
							.add_filter("Executable", &["exe"])
							.set_file_name("program_hybrid.exe")
							.save_file()
						{
							match lang::build_true_hybrid(&self.source_code, &path) {
								Ok(info) => {
									self.compiler_output = info;
									self.status_msg = "✅ Hybrid EXE".into();
									self.error_lines.clear();
								}
								Err(e) => {
									self.compiler_output = format!("❌ {}", e);
									self.status_msg = "Error".into();
									self.error_lines = Self::parse_error_lines(&self.source_code, &e);
								}
							}
						}
						ui.close_menu();
					}
					
					ui.separator();
					ui.collapsing("❓ Требования", |ui| {
						ui.label("1️⃣ NASM:");
						if ui.small_button("📥 nasm.us").clicked() {
							let _ = open::that("https://www.nasm.us/");
						}
						ui.label("2️⃣ Линкер:");
						if ui.small_button("📥 GoLink").clicked() {
							let _ = open::that("https://www.godevtool.com/");
						}
						if ui.small_button("📥 VS Build Tools").clicked() {
							let _ = open::that("https://visualstudio.microsoft.com/visual-cpp-build-tools/");
						}
						ui.label("3️⃣ Rust (для гибрида):");
						if ui.small_button("📥 rustup.rs").clicked() {
							let _ = open::that("https://rustup.rs/");
						}
					});
				}
			);
			if ui.button(self.texts().installer_msi).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Installer", &["exe", "msi"])
                    .set_file_name("program_setup.exe")
                    .save_file()
                {
                    match lang::build_msi(&self.source_code, "program", &path) {
                        Ok(info) => {
                            self.compiler_output = info;
                            self.status_msg = format!("✅ Installer: {}", path.display());
							self.error_lines.clear();
                        }
                        Err(e) => {
                            self.compiler_output = format!("❌ {}", e);
                            self.status_msg = "Somthing went wrong...".into();
							self.error_lines = Self::parse_error_lines(&self.source_code, &e);
                        }
                    }
                }
            }
            if ui.button(self.texts().show_rust).clicked() {
                match lang::transpile_to_rust(&self.source_code) {
                    Ok(rust_code) => {
                        self.bytecode_view = rust_code;
                        self.show_bytecode = true;
                        self.status_msg = "Rust code generated!".into();
						self.error_lines.clear();
                    }
                    Err(e) => {
                        self.status_msg = format!("Error: {}", e);
						self.error_lines = Self::parse_error_lines(&self.source_code, &e);
                    }
                }
            }
			ui.separator();
			if ui.button(self.texts().open).clicked() {
				self.load_source();
			}
			if ui.button(self.texts().save_source).clicked() {
				self.save_source();
			}
			ui.separator();

			// === НОВОЕ: Undo / Redo ===
			let undo_label = self.texts().undo;
			let redo_label = self.texts().redo;
			let has_undo = !self.undo_stack.is_empty();
			let has_redo = !self.redo_stack.is_empty();

			if ui.add_enabled(has_undo, egui::Button::new(undo_label)).clicked() {
				self.undo();
			}
			if ui.add_enabled(has_redo, egui::Button::new(redo_label)).clicked() {
				self.redo();
			}
			ui.separator();
			// === КОНЕЦ Undo / Redo ===

			let bytecode_label = self.texts().bytecode;
			ui.checkbox(&mut self.show_bytecode, bytecode_label);
			ui.separator();

			ui.menu_button("📚 Examples", |ui| {
				if ui.button("Hello World").clicked() {
					self.source_code = r#"print("Hello, Rust--!");"#.into();
					ui.close_menu();
				}
				if ui.button("Counter").clicked() {
					self.source_code = r#"let i = 1;
	loop {
		if i > 10 {
			break;
		}
		print(i);
		i = i + 1;
	}"#
					.into();
					ui.close_menu();
				}
				if ui.button("idk how to name this").clicked() {
					self.source_code = r#"let a = 0;
	let b = 1;
	let i = 0;
	loop {
		if i >= 15 {
			break;
		}
		print("fib(" + i + ") = " + a);
		let temp = b;
		b = a + b;
		a = temp;
		i = i + 1;
	}"#
					.into();
					ui.close_menu();
				}
				if ui.button("FizzBuzz").clicked() {
					self.source_code = r#"let i = 1;
	loop {
		if i > 30 {
			break;
		}
		if i % 15 == 0 {
			print("FizzBuzz");
		} else {
			if i % 3 == 0 {
				print("Fizz");
			} else {
				if i % 5 == 0 {
					print("Buzz");
				} else {
					print(i);
				}
			}
		}
		i = i + 1;
	}"#
					.into();
					ui.close_menu();
				}
				if ui.button("Functions").clicked() {
					self.source_code = r#"fn greet(name) {
		print("Привет, " + name + "!");
	}

	fn square(n) {
		let result = n * n;
		print(n + "² = " + result);
	}

	greet("Мир");
	greet("Rust--");
	square(5);
	square(12);
	square(100);"#
						.into();
					ui.close_menu();
				}
			});
		});

		ui.separator();

		// Главная область — используем columns чтобы занять ВСЮ высоту
		let num_cols = if self.show_bytecode { 3 } else { 2 };

		ui.columns(num_cols, |cols| {
			// Колонка 1: Исходный код
			cols[0].label(
				egui::RichText::new("📝 Code (.rsmm)")
					.strong()
					.color(egui::Color32::from_rgb(255, 200, 100)),
			);
			syntax_highlight::code_editor(&mut cols[0], &mut self.source_code, "source_editor", &self.error_lines);

			if self.show_bytecode {
				// Колонка 2: Байткод
				cols[1].label(
					egui::RichText::new("📋 Bitecode")
						.strong()
						.color(egui::Color32::from_rgb(150, 200, 255)),
				);
				let mut bytecode_clone = self.bytecode_view.clone();
				egui::ScrollArea::vertical()
					.id_source("bytecode_scroll")
					.show(&mut cols[1], |ui| {
						ui.add(
							egui::TextEdit::multiline(&mut bytecode_clone)
								.code_editor()
								.desired_width(f32::INFINITY)
								.desired_rows(40),
						);
					});

				// Колонка 3: Вывод
				self.render_output_column(&mut cols[2]);
			} else {
				// Колонка 2: Вывод
				self.render_output_column(&mut cols[1]);
			}
		});
	}

	fn render_output_column(&mut self, ui: &mut egui::Ui) {
		// Compiler output
		ui.label(
			egui::RichText::new("🔨 Compile")
				.strong()
				.color(egui::Color32::from_rgb(200, 255, 200)),
		);

		let color = if self.compiler_output.contains('❌') {
			egui::Color32::from_rgb(255, 100, 100)
		} else {
			egui::Color32::from_rgb(100, 255, 100)
		};
		ui.colored_label(color, &self.compiler_output);

		if !self.run_output.is_empty() {
			ui.separator();
			ui.label(
				egui::RichText::new("▶ Output")
					.strong()
					.color(egui::Color32::from_rgb(100, 200, 255)),
			);

			egui::ScrollArea::vertical()
				.id_source("run_output_scroll")
				.show(ui, |ui| {
					let color = if self.run_output.contains('❌') {
						egui::Color32::from_rgb(255, 100, 100)
					} else {
						egui::Color32::from_rgb(220, 220, 220)
					};

					for (i, line) in self.run_output.lines().enumerate() {
						ui.horizontal(|ui| {
							ui.label(
								egui::RichText::new(format!("{:>3}│", i + 1))
									.color(egui::Color32::from_rgb(100, 100, 100))
									.monospace(),
							);
							ui.label(
								egui::RichText::new(line).color(color).monospace(),
							);
						});
					}
				});
		}
	}

    fn render_decompiler_tab(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new("📂 Load .rsm")
                        .color(egui::Color32::from_rgb(100, 200, 255)),
                )
                .clicked()
            {
                self.load_for_decompile();
            }
            if ui.button("💾 Save as .rsmm").clicked() {
                self.save_decompiled();
            }
            ui.separator();
            if ui.button("📋 Copy in compilator").clicked() {
                if !self.decompiled_source.is_empty() {
                    self.source_code = self.decompiled_source.clone();
                    self.active_tab = Tab::Compiler;
                    self.status_msg = "Copied!".into();
                }
            }
        });

        ui.separator();

        // Status
        let status_color = if self.decompiler_status.contains('❌') {
            egui::Color32::from_rgb(255, 100, 100)
        } else if self.decompiler_status.contains('✅') {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(200, 200, 200)
        };
        ui.colored_label(status_color, &self.decompiler_status);

        ui.separator();

        // Decompiled source
        let available = ui.available_size();
        ui.label(
            egui::RichText::new("🔍 Decompiled code Rust--")
                .strong()
                .color(egui::Color32::from_rgb(255, 200, 100)),
        );
		let no_errors = std::collections::HashSet::new();
		syntax_highlight::code_editor(ui, &mut self.decompiled_source, "decompiled_editor", &no_errors);
    }
}
