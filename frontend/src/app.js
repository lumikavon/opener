// Opener - Desktop Launcher Application
// Main Frontend Application

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ==================== State Management ====================

const state = {
  entries: [],
  hotkeys: [],
  templates: [],
  settings: null,
  searchResults: [],
  selectedIndex: 0,
  currentTab: 'entries',
  searchDebounceTimer: null,
  entriesSearchQuery: '',
  selectedEntryIds: new Set(),
};

const DEFAULT_LANGUAGE = 'zh-CN';
const GITHUB_REPO_URL = 'https://github.com/lumikavon/opener';

const translations = {
  'zh-CN': {
    app: {
      name: 'Opener',
      version: '1.0.0',
      author: 'virtualink',
      wechat: 'virtualink',
    },
    title_bar: {
      minimize: '最小化',
      maximize: '最大化',
      close: '关闭',
      github: 'GitHub',
      help: '帮助',
      about: '关于',
    },
    search: {
      placeholder: '搜索...',
      empty: '没有匹配结果',
    },
    settings: {
      title: '设置',
        sidebar: {
          entries: '条目',
          display: '显示',
          templates: '模板',
          import_export: '导入/导出',
          about: '关于',
        },
      entries: {
        title: '管理条目',
        add: '新增条目',
        empty: '暂无条目。点击“新增条目”创建。',
        search_placeholder: '搜索条目...',
        no_match: '没有匹配的条目。',
        select_all: '全选',
        selected_count: '已选择 {{count}} 个',
        select_entry: '选择 {{name}}',
        copy_suffix: '副本',
        untitled: '未命名',
      },
        display: {
        title: '显示与排序',
        icon_size: '图标大小',
        sort_by: '排序方式',
        max_results: '最大结果数',
        language: '语言',
        sort: {
          relevance: '相关性',
          name: '名称',
          recently_used: '最近使用',
          use_count: '最常使用',
        },
        show_path: '显示路径/目标',
        show_type: '显示类型标签',
        show_description: '显示描述',
        confirm_dangerous: '运行危险命令前确认（Cmd/WSL/SSH/脚本/AHK）',
        auto_launch: '开机自启动',
        app_hotkey: '快捷键（切换显示/隐藏）',
        app_hotkey_hint: '点击并按下快捷键，用于切换 Opener 显示与隐藏。',
        app_hotkey_placeholder: '按下组合键...',
        language_options: {
          zh: '简体中文',
          en: 'English',
        },
      },
      templates: {
        title: '脚本模板',
        add: '新增模板',
        empty: '暂无模板。点击“新增模板”创建。',
      },
      import_export: {
        title: '导入与导出',
        export_title: '导出配置',
        export_desc: '导出所有条目、快捷键、设置和模板到 JSON 文件。',
        export_button: '导出到文件',
        import_title: '导入配置',
        import_desc: '从之前导出的 JSON 文件导入配置。',
        import_strategy: '导入策略',
        import_options: {
          add_only: '仅新增（跳过已存在）',
          merge_by_name: '按名称合并（更新已存在）',
          overwrite_all: '全部覆盖（替换所有内容）',
        },
        import_button: '从文件导入',
      },
      about: {
        title: '关于 Opener',
        subtitle: '桌面启动器应用',
        version: '版本 1.0.0',
        built_with: '基于 Tauri v2、HTML5、TailwindCSS 和 SQLite 构建',
        offline: '所有依赖已本地打包 - 无需联网',
      },
    },
    about_modal: {
      title: '关于',
      labels: {
        name: '软件名称',
        version: '版本',
        author: '作者',
        wechat: '作者微信',
      },
    },
    help_modal: {
      title: '帮助',
      sections: {
        overview: {
          title: '概览',
          body: 'Opener 是一款离线可用的桌面启动器，支持多种条目类型与快捷键操作。',
        },
        features: {
          title: '主要功能',
          item1: '支持 App / URL / 文件 / 目录 / 命令 / WSL / SSH / 脚本 / 快捷方式 / AutoHotkey',
          item2: '模糊搜索与键盘导航',
          item3: '脚本模板与变量替换',
          item4: '快捷键绑定（应用级与全局）',
          item5: '导入/导出配置',
          item6: '敏感信息安全存储，离线运行',
        },
        add_entry: {
          title: '新增条目',
          item1: '打开设置（齿轮按钮）',
          item2: '进入“条目”标签页',
          item3: '点击“新增条目”',
          item4: '选择类型并填写对应字段',
        },
        shortcuts: {
          title: '键盘操作',
          item1: '↑/↓：在搜索结果中移动',
          item2: 'Enter：执行选中条目',
          item3: 'Esc：清空搜索或关闭弹窗',
        },
        templates: {
          title: '脚本模板',
          body: '进入“模板”标签页，点击使用按钮，填写变量后即可生成条目。模板变量使用 {{variable_name}} 语法。',
        },
        import_export: {
          title: '导入/导出',
          body: '在“导入/导出”中可导出全部配置，或从 JSON 文件导入并选择合并策略。',
        },
        requirements: {
          title: '依赖说明',
          body: 'Windows 下使用 AutoHotkey 功能需要安装 AutoHotkey v2。',
          ahk_install: '可通过 winget 安装：winget install AutoHotkey.AutoHotkey',
        },
      },
    },
    entry: {
      modal_add: '新增条目',
      modal_edit: '编辑条目',
      fields: {
        name: '名称 *',
        type: '类型 *',
        target: '目标 *',
        app_name: '应用名称 *',
        hotkey: '快捷键 *',
        args: '参数',
        workdir: '工作目录（支持 ${VAR} 环境变量）',
        description: '描述',
        tags: '标签',
        global_hotkey: '全局快捷键（可选）',
        icon_path: '图标路径',
        hotkey_filter: '窗口匹配 *（支持 ${VAR} 环境变量）',
        hotkey_position: '窗口位置 *',
        hotkey_detect_hidden: '检测隐藏窗口',
        ssh_host: 'SSH 主机 *（支持 ${VAR} 环境变量）',
        ssh_user: 'SSH 用户（支持 ${VAR} 环境变量）',
        ssh_port: 'SSH 端口',
        wsl_distro: 'WSL 发行版',
        script_content: '脚本内容',
        show_terminal: '显示终端窗口',
        require_confirm: '需要确认',
      },
      placeholders: {
        args: '可选参数',
        description: '可选描述',
        tags: '逗号分隔的标签',
        global_hotkey: '按下组合键...',
        hotkey_filter: '窗口标题或 ahk_exe 程序名',
        ssh_host: '主机名或 IP',
        ssh_user: 'root',
        wsl_distro: 'Ubuntu',
        script_content: '输入脚本内容...',
      },
      helpers: {
        global_hotkey: '点击并按下所需组合键，留空则不绑定。',
        hotkey_required: '点击并按下组合键（必填）。',
      },
      target_labels: {
        url: 'URL *',
        file: '文件路径 *（支持 ${VAR} 环境变量）',
        dir: '目录路径 *（支持 ${VAR} 环境变量）',
        command: '命令 *（支持 ${VAR} 环境变量）',
        wsl_command: '命令 *（支持 ${VAR} 环境变量）',
        script_path: '脚本路径（或使用下方内容，支持 ${VAR} 环境变量）',
        executable: '可执行文件 *（支持 ${VAR} 环境变量）',
        target: '目标 *',
      },
      target_placeholders: {
        url: 'https://example.com',
        file: '文件路径',
        dir: '目录路径',
        command: '要执行的命令',
        wsl_command: '要在 WSL 中执行的命令',
        script_path: '脚本文件路径（可选）',
        executable: '可执行文件路径',
      },
      type_options: {
        app: '应用',
        url: 'URL',
        file: '文件',
        dir: '目录',
        cmd: '命令',
        wsl: 'WSL',
        ssh: 'SSH',
        script: '脚本',
        shortcut: '快捷方式 (.lnk)',
        ahk: 'AutoHotkey',
        hotkey_app: 'HotKey应用',
      },
      type_labels: {
        app: '应用',
        url: 'URL',
        file: '文件',
        dir: '目录',
        cmd: '命令',
        wsl: 'WSL',
        ssh: 'SSH',
        script: '脚本',
        shortcut: '快捷方式',
        ahk: 'AHK',
        hotkey_app: 'HOTKEY',
      },
      position_options: {
        keep: '保持不变',
        left: '左侧',
        right: '右侧',
        max: '最大化',
      },
      created_from_template: '由模板创建：{{name}}',
      disabled: '（已禁用）',
    },
    template: {
      modal_add: '新增模板',
      modal_edit: '编辑模板',
      modal_use: '使用模板',
      modal_use_named: '使用模板：{{name}}',
      fields: {
        name: '名称 *',
        description: '描述',
        language: '语言/类型',
        content: '模板内容 *',
        variables: '变量',
        entry_name: '条目名称 *',
        preview: '预览',
      },
      placeholders: {
        content: '使用 {{variable_name}} 表示变量',
        var_name: '变量名',
        var_label: '标签',
        var_default: '默认值',
      },
      helper: {
        content_hint: '使用 {{variable_name}} 语法表示变量',
      },
      add_variable: '+ 添加变量',
      variable_label: '变量 {{index}}',
      remove_variable: '移除',
      required: '必填',
      create_entry: '创建条目',
      empty: '暂无模板。点击“新增模板”创建。',
      languages: {
        shell: 'Shell (bash/sh)',
        powershell: 'PowerShell',
        cmd: 'CMD (Windows)',
        python: 'Python',
        ahk: 'AutoHotkey',
        ssh: 'SSH 命令',
        wsl: 'WSL 命令',
      },
      variable_types: {
        string: '字符串',
        number: '数字',
        path: '路径',
        choice: '选项',
        boolean: '布尔',
      },
    },
    confirm: {
      title: '确认操作',
      cancel: '取消',
      execute: '执行',
      delete_entry_title: '删除条目',
      delete_entry_message: '确定要删除“{{name}}”吗？',
      delete_entries_title: '删除条目',
      delete_entries_message: '确定要删除选中的 {{count}} 个条目吗？',
      delete_template_title: '删除模板',
      delete_template_message: '确定要删除“{{name}}”吗？',
      execute_command_title: '执行命令',
      execute_command_message: '确定要执行此 {{type}} 吗？',
      irreversible: '此操作无法撤销。',
    },
    context_menu: {
      execute: '执行',
      edit: '编辑',
      copy_path: '复制路径',
      delete: '删除',
    },
    actions: {
      browse: '浏览',
      cancel: '取消',
      clear: '清除',
      save: '保存',
      test: '测试',
      create_entry: '创建条目',
      edit: '编辑',
      duplicate: '复制',
      delete: '删除',
      delete_selected: '删除所选',
      use_template: '使用模板',
      execute: '执行',
      window_spy: '窗口匹配',
    },
    labels: {
      disabled: '（已禁用）',
    },
    toasts: {
      search_failed: '搜索失败：{{error}}',
      load_entries_failed: '加载条目失败',
      entry_created: '条目创建成功',
      entry_create_failed: '创建条目失败：{{error}}',
      entry_updated: '条目更新成功',
      entry_update_failed: '更新条目失败：{{error}}',
      entry_deleted: '条目已删除',
      entry_delete_failed: '删除条目失败：{{error}}',
      entries_deleted: '已删除 {{count}} 个条目',
      entries_delete_failed: '删除失败：{{count}} 个条目未能删除',
      executed: '已执行：{{preview}}',
      execution_failed: '执行失败：{{error}}',
      settings_save_failed: '保存设置失败',
      hotkey_created: '快捷键已创建',
      hotkey_create_failed: '创建快捷键失败：{{error}}',
      hotkey_updated: '快捷键已更新',
      hotkey_update_failed: '更新快捷键失败：{{error}}',
      hotkey_deleted: '快捷键已删除',
      hotkey_delete_failed: '删除快捷键失败',
      template_created: '模板创建成功',
      template_create_failed: '创建模板失败：{{error}}',
      template_updated: '模板更新成功',
      template_update_failed: '更新模板失败：{{error}}',
      template_deleted: '模板已删除',
      template_delete_failed: '删除模板失败',
      export_success: '配置导出成功',
      export_failed: '导出失败：{{error}}',
      import_complete: '导入完成：{{entries}} 个条目，{{hotkeys}} 个快捷键，{{templates}} 个模板',
      import_failed: '导入失败：{{error}}',
      entry_from_template: '已从模板创建条目',
      copy_path: '路径已复制到剪贴板',
      window_spy_failed: '窗口匹配启动失败：{{error}}',
    },
  },
  'en-US': {
    app: {
      name: 'Opener',
      version: '1.0.0',
      author: 'virtualink',
      wechat: 'virtualink',
    },
    title_bar: {
      minimize: 'Minimize',
      maximize: 'Maximize',
      close: 'Close',
      github: 'GitHub',
      help: 'Help',
      about: 'About',
    },
    search: {
      placeholder: 'Search...',
      empty: 'No matching results',
    },
    settings: {
      title: 'Settings',
        sidebar: {
          entries: 'Entries',
          display: 'Display',
          templates: 'Templates',
          import_export: 'Import/Export',
          about: 'About',
        },
      entries: {
        title: 'Manage Entries',
        add: 'Add Entry',
        empty: 'No entries yet. Click "Add Entry" to create one.',
        search_placeholder: 'Search entries...',
        no_match: 'No matching entries.',
        select_all: 'Select all',
        selected_count: 'Selected {{count}} items',
        select_entry: 'Select {{name}}',
        copy_suffix: 'Copy',
        untitled: 'Untitled',
      },
        display: {
        title: 'Display & Sorting',
        icon_size: 'Icon Size',
        sort_by: 'Sort By',
        max_results: 'Max Results',
        language: 'Language',
        sort: {
          relevance: 'Relevance',
          name: 'Name',
          recently_used: 'Recently Used',
          use_count: 'Most Used',
        },
        show_path: 'Show path/target',
        show_type: 'Show type label',
        show_description: 'Show description',
        confirm_dangerous: 'Confirm before running dangerous commands (Cmd/WSL/SSH/Script/AHK)',
        auto_launch: 'Launch on system startup',
        app_hotkey: 'Hotkey (toggle show/hide)',
        app_hotkey_hint: 'Click and press a hotkey to toggle Opener visibility.',
        app_hotkey_placeholder: 'Press keys...',
        language_options: {
          zh: '简体中文',
          en: 'English',
        },
      },
      templates: {
        title: 'Script Templates',
        add: 'Add Template',
        empty: 'No templates yet. Click "Add Template" to create one.',
      },
      import_export: {
        title: 'Import & Export',
        export_title: 'Export Configuration',
        export_desc: 'Export all entries, hotkeys, settings, and templates to a JSON file.',
        export_button: 'Export to File',
        import_title: 'Import Configuration',
        import_desc: 'Import configuration from a previously exported JSON file.',
        import_strategy: 'Import Strategy',
        import_options: {
          add_only: 'Add only (skip existing)',
          merge_by_name: 'Merge by name (update existing)',
          overwrite_all: 'Overwrite all (replace everything)',
        },
        import_button: 'Import from File',
      },
      about: {
        title: 'About Opener',
        subtitle: 'Desktop Launcher Application',
        version: 'Version 1.0.0',
        built_with: 'Built with Tauri v2, HTML5, TailwindCSS, and SQLite',
        offline: 'All dependencies are bundled locally - no internet required',
      },
    },
    about_modal: {
      title: 'About',
      labels: {
        name: 'App Name',
        version: 'Version',
        author: 'Author',
        wechat: 'WeChat',
      },
    },
    help_modal: {
      title: 'Help',
      sections: {
        overview: {
          title: 'Overview',
          body: 'Opener is an offline-capable desktop launcher that supports multiple entry types and hotkeys.',
        },
        features: {
          title: 'Key Features',
          item1: 'Supports App / URL / File / Directory / Command / WSL / SSH / Script / Shortcut / AutoHotkey',
          item2: 'Fuzzy search with keyboard navigation',
          item3: 'Script templates with variable substitution',
          item4: 'Hotkey bindings (app-level and global)',
          item5: 'Import/Export configuration',
          item6: 'Secure storage and fully offline operation',
        },
        add_entry: {
          title: 'Add Entry',
          item1: 'Open Settings (gear button)',
          item2: 'Go to the “Entries” tab',
          item3: 'Click “Add Entry”',
          item4: 'Choose a type and fill in the fields',
        },
        shortcuts: {
          title: 'Keyboard',
          item1: '↑/↓: move in search results',
          item2: 'Enter: execute selected entry',
          item3: 'Esc: clear search or close modal',
        },
        templates: {
          title: 'Script Templates',
          body: 'Open the “Templates” tab, click Use, fill variables, and create an entry. Variables use {{variable_name}} syntax.',
        },
        import_export: {
          title: 'Import/Export',
          body: 'Export all configuration or import from JSON with a merge strategy.',
        },
        requirements: {
          title: 'Requirements',
          body: 'AutoHotkey v2 is required on Windows for AHK entries.',
          ahk_install: 'Install via winget: winget install AutoHotkey.AutoHotkey',
        },
      },
    },
    entry: {
      modal_add: 'Add Entry',
      modal_edit: 'Edit Entry',
      fields: {
        name: 'Name *',
        type: 'Type *',
        target: 'Target *',
        app_name: 'App Name *',
        hotkey: 'Hotkey *',
        args: 'Arguments',
        workdir: 'Working Directory (supports ${VAR})',
        description: 'Description',
        tags: 'Tags',
        global_hotkey: 'Global Hotkey (optional)',
        icon_path: 'Icon Path',
        hotkey_filter: 'Window Filter * (supports ${VAR})',
        hotkey_position: 'Window Position *',
        hotkey_detect_hidden: 'Detect hidden windows',
        ssh_host: 'SSH Host * (supports ${VAR})',
        ssh_user: 'SSH User (supports ${VAR})',
        ssh_port: 'SSH Port',
        wsl_distro: 'WSL Distribution',
        script_content: 'Script Content',
        show_terminal: 'Show terminal window',
        require_confirm: 'Require confirmation',
      },
      placeholders: {
        args: 'Optional arguments',
        description: 'Optional description',
        tags: 'Comma-separated tags',
        global_hotkey: 'Press keys...',
        hotkey_filter: 'Window title or ahk_exe process',
        ssh_host: 'hostname or IP',
        ssh_user: 'root',
        wsl_distro: 'Ubuntu',
        script_content: 'Enter script content...',
      },
      helpers: {
        global_hotkey: 'Click and press a key combination. Leave empty to skip.',
        hotkey_required: 'Click and press a key combination (required).',
      },
      target_labels: {
        url: 'URL *',
        file: 'File Path * (supports ${VAR})',
        dir: 'Directory Path * (supports ${VAR})',
        command: 'Command * (supports ${VAR})',
        wsl_command: 'Command * (supports ${VAR})',
        script_path: 'Script Path (or use content below, supports ${VAR})',
        executable: 'Executable * (supports ${VAR})',
        target: 'Target *',
      },
      target_placeholders: {
        url: 'https://example.com',
        file: 'Path to file',
        dir: 'Path to directory',
        command: 'Command to execute',
        wsl_command: 'Command to execute in WSL',
        script_path: 'Path to script file (optional)',
        executable: 'Path to executable',
      },
      type_options: {
        app: 'Application',
        url: 'URL',
        file: 'File',
        dir: 'Directory',
        cmd: 'Command',
        wsl: 'WSL',
        ssh: 'SSH',
        script: 'Script',
        shortcut: 'Shortcut (.lnk)',
        ahk: 'AutoHotkey',
        hotkey_app: 'HotKey App',
      },
      type_labels: {
        app: 'APP',
        url: 'URL',
        file: 'FILE',
        dir: 'DIR',
        cmd: 'CMD',
        wsl: 'WSL',
        ssh: 'SSH',
        script: 'SCRIPT',
        shortcut: 'SHORTCUT',
        ahk: 'AHK',
        hotkey_app: 'HOTKEY',
      },
      position_options: {
        keep: 'Keep',
        left: 'Left',
        right: 'Right',
        max: 'Maximize',
      },
      created_from_template: 'Created from template: {{name}}',
      disabled: '(disabled)',
    },
    template: {
      modal_add: 'Add Template',
      modal_edit: 'Edit Template',
      modal_use: 'Use Template',
      modal_use_named: 'Use Template: {{name}}',
      fields: {
        name: 'Name *',
        description: 'Description',
        language: 'Language/Type',
        content: 'Template Content *',
        variables: 'Variables',
        entry_name: 'Entry Name *',
        preview: 'Preview',
      },
      placeholders: {
        content: 'Use {{variable_name}} for variables',
        var_name: 'Variable name',
        var_label: 'Label',
        var_default: 'Default value',
      },
      helper: {
        content_hint: 'Use {{variable_name}} syntax for variables',
      },
      add_variable: '+ Add Variable',
      variable_label: 'Variable {{index}}',
      remove_variable: 'Remove',
      required: 'Required',
      create_entry: 'Create Entry',
      empty: 'No templates yet. Click "Add Template" to create one.',
      languages: {
        shell: 'Shell (bash/sh)',
        powershell: 'PowerShell',
        cmd: 'CMD (Windows)',
        python: 'Python',
        ahk: 'AutoHotkey',
        ssh: 'SSH Command',
        wsl: 'WSL Command',
      },
      variable_types: {
        string: 'String',
        number: 'Number',
        path: 'Path',
        choice: 'Choice',
        boolean: 'Boolean',
      },
    },
    confirm: {
      title: 'Confirm Action',
      cancel: 'Cancel',
      execute: 'Execute',
      delete_entry_title: 'Delete Entry',
      delete_entry_message: 'Are you sure you want to delete "{{name}}"?',
      delete_entries_title: 'Delete Entries',
      delete_entries_message: 'Are you sure you want to delete the selected {{count}} entries?',
      delete_template_title: 'Delete Template',
      delete_template_message: 'Are you sure you want to delete "{{name}}"?',
      execute_command_title: 'Execute Command',
      execute_command_message: 'Are you sure you want to execute this {{type}} command?',
      irreversible: 'This action cannot be undone.',
    },
    context_menu: {
      execute: 'Execute',
      edit: 'Edit',
      copy_path: 'Copy Path',
      delete: 'Delete',
    },
    actions: {
      browse: 'Browse',
      cancel: 'Cancel',
      clear: 'Clear',
      save: 'Save',
      test: 'Test',
      create_entry: 'Create Entry',
      edit: 'Edit',
      duplicate: 'Duplicate',
      delete: 'Delete',
      delete_selected: 'Delete Selected',
      use_template: 'Use Template',
      execute: 'Execute',
      window_spy: 'Window Match',
    },
    labels: {
      disabled: '(disabled)',
    },
    toasts: {
      search_failed: 'Search failed: {{error}}',
      load_entries_failed: 'Failed to load entries',
      entry_created: 'Entry created successfully',
      entry_create_failed: 'Failed to create entry: {{error}}',
      entry_updated: 'Entry updated successfully',
      entry_update_failed: 'Failed to update entry: {{error}}',
      entry_deleted: 'Entry deleted',
      entry_delete_failed: 'Failed to delete entry: {{error}}',
      entries_deleted: 'Deleted {{count}} entries',
      entries_delete_failed: 'Failed to delete {{count}} entries',
      executed: 'Executed: {{preview}}',
      execution_failed: 'Execution failed: {{error}}',
      settings_save_failed: 'Failed to save settings',
      hotkey_created: 'Hotkey created',
      hotkey_create_failed: 'Failed to create hotkey: {{error}}',
      hotkey_updated: 'Hotkey updated',
      hotkey_update_failed: 'Failed to update hotkey: {{error}}',
      hotkey_deleted: 'Hotkey deleted',
      hotkey_delete_failed: 'Failed to delete hotkey',
      template_created: 'Template created',
      template_create_failed: 'Failed to create template: {{error}}',
      template_updated: 'Template updated',
      template_update_failed: 'Failed to update template: {{error}}',
      template_deleted: 'Template deleted',
      template_delete_failed: 'Failed to delete template',
      export_success: 'Configuration exported successfully',
      export_failed: 'Export failed: {{error}}',
      import_complete: 'Import complete: {{entries}} entries, {{hotkeys}} hotkeys, {{templates}} templates',
      import_failed: 'Import failed: {{error}}',
      entry_from_template: 'Entry created from template',
      copy_path: 'Path copied to clipboard',
      window_spy_failed: 'Failed to open WindowSpy: {{error}}',
    },
  },
};

// ==================== Utilities ====================

function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

function showToast(message, type = 'info') {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');

  const colors = {
    info: 'bg-primary-600',
    success: 'bg-green-600',
    error: 'bg-red-600',
    warning: 'bg-yellow-600',
  };

  toast.className = `${colors[type]} text-white px-4 py-3 rounded-lg shadow-lg animate-slide-down flex items-center gap-2`;
  toast.innerHTML = `
    <span>${message}</span>
    <button class="ml-2 hover:opacity-75" data-toast-action="close">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
      </svg>
    </button>
  `;

  container.appendChild(toast);
  const closeButton = toast.querySelector('[data-toast-action="close"]');
  if (closeButton) {
    closeButton.addEventListener('click', () => toast.remove());
  }
  setTimeout(() => toast.remove(), 5000);
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function buildDuplicateName(name) {
  const suffix = t('settings.entries.copy_suffix');
  const baseName = (name || '').trim() || t('settings.entries.untitled');
  if (!suffix) {
    return `${baseName} Copy`;
  }
  const pattern = new RegExp(`\\s*\\(${escapeRegExp(suffix)}(?:\\s*(\\d+))?\\)\\s*$`);
  const match = baseName.match(pattern);
  if (!match) {
    return `${baseName} (${suffix})`;
  }
  const count = match[1] ? Number(match[1]) + 1 : 2;
  const stripped = baseName.replace(pattern, '').trim();
  return `${stripped} (${suffix} ${count})`;
}

async function openExternalLink(url) {
  try {
    if (window.__TAURI__?.shell?.open) {
      await window.__TAURI__.shell.open(url);
      return;
    }
  } catch (error) {
    // Fallback to window.open below.
  }

  try {
    window.open(url, '_blank', 'noopener');
  } catch (error) {
    // Ignore if blocked.
  }
}

function normalizeLanguage(language) {
  if (!language) return DEFAULT_LANGUAGE;
  const lower = language.toLowerCase();
  if (lower.startsWith('zh')) return 'zh-CN';
  if (lower.startsWith('en')) return 'en-US';
  return DEFAULT_LANGUAGE;
}

function getCurrentLanguage() {
  return normalizeLanguage(state.settings?.language || DEFAULT_LANGUAGE);
}

function getTranslationTable(language) {
  const normalized = normalizeLanguage(language || getCurrentLanguage());
  return translations[normalized] || translations[DEFAULT_LANGUAGE];
}

function getTranslationValue(language, keyPath) {
  return keyPath.split('.').reduce((acc, key) => (acc ? acc[key] : undefined), getTranslationTable(language));
}

function t(keyPath, params = {}, language) {
  const normalized = normalizeLanguage(language || getCurrentLanguage());
  let value = getTranslationValue(normalized, keyPath);
  if (value === undefined) {
    value = getTranslationValue(DEFAULT_LANGUAGE, keyPath);
  }
  if (typeof value !== 'string') {
    return keyPath;
  }
  return Object.keys(params).reduce((text, key) => text.replaceAll(`{{${key}}}`, params[key]), value);
}

function applyTranslations(language) {
  const normalized = normalizeLanguage(language || DEFAULT_LANGUAGE);
  document.documentElement.lang = normalized;

  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.dataset.i18n;
    el.textContent = t(key, {}, normalized);
  });

  document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
    const key = el.dataset.i18nPlaceholder;
    el.placeholder = t(key, {}, normalized);
  });

  document.querySelectorAll('[data-i18n-title]').forEach((el) => {
    const key = el.dataset.i18nTitle;
    el.title = t(key, {}, normalized);
  });
}

  function applyLanguage(language) {
    const normalized = normalizeLanguage(language || DEFAULT_LANGUAGE);
    applyTranslations(normalized);
    updateEntryFormFields();
    renderEntriesList(state.entries);
    renderTemplatesList(state.templates);
    renderSearchResults(state.searchResults);
  }

function getEntryTypeLabel(type) {
  const labels = getTranslationTable().entry?.type_labels || {};
  return labels[type] || type.toUpperCase();
}

function getTemplateLanguageLabel(language) {
  const labels = getTranslationTable().template?.languages || {};
  return labels[language] || language.toUpperCase();
}

// ==================== API Calls ====================

async function searchEntries(query) {
  try {
    const results = await invoke('search_entries', { query });
    return results;
  } catch (error) {
    console.error('Search failed:', error);
    showToast(t('toasts.search_failed', { error: error.message }), 'error');
    return [];
  }
}

async function getAllEntries() {
  try {
    return await invoke('get_all_entries');
  } catch (error) {
    console.error('Failed to get entries:', error);
    showToast(t('toasts.load_entries_failed'), 'error');
    return [];
  }
}

async function createEntry(input) {
  try {
    const entry = await invoke('create_entry', { input });
    showToast(t('toasts.entry_created'), 'success');
    return entry;
  } catch (error) {
    console.error('Failed to create entry:', error);
    showToast(t('toasts.entry_create_failed', { error: error.message }), 'error');
    throw error;
  }
}

async function updateEntry(id, input) {
  try {
    const entry = await invoke('update_entry', { id, input });
    showToast(t('toasts.entry_updated'), 'success');
    return entry;
  } catch (error) {
    console.error('Failed to update entry:', error);
    showToast(t('toasts.entry_update_failed', { error: error.message }), 'error');
    throw error;
  }
}

async function deleteEntry(id, options = {}) {
  const silent = options.silent === true;
  try {
    await invoke('delete_entry', { id });
    if (!silent) {
      showToast(t('toasts.entry_deleted'), 'success');
    }
  } catch (error) {
    console.error('Failed to delete entry:', error);
    if (!silent) {
      showToast(t('toasts.entry_delete_failed', { error: error.message }), 'error');
    }
    throw error;
  }
}

async function executeEntry(id, options = {}) {
  const silent = options.silent === true;
  try {
    const preview = await invoke('execute_entry', { id, ahkPath: null });
    if (!silent) {
      showToast(t('toasts.executed', { preview }), 'success');
    }
    return preview;
  } catch (error) {
    if (!silent) {
      console.error('Execution failed:', error);
      showToast(t('toasts.execution_failed', { error: error.message }), 'error');
      throw error;
    }
    console.error('Execution failed (silent):', error);
    return null;
  }
}

async function executeEntryInput(input, options = {}) {
  const silent = options.silent === true;
  try {
    const preview = await invoke('execute_entry_input', { input, ahkPath: null });
    if (!silent) {
      showToast(t('toasts.executed', { preview }), 'success');
    }
    return preview;
  } catch (error) {
    if (!silent) {
      console.error('Execution failed:', error);
      showToast(t('toasts.execution_failed', { error: error.message }), 'error');
      throw error;
    }
    console.error('Execution failed (silent):', error);
    return null;
  }
}

async function getSettings() {
  try {
    return await invoke('get_settings');
  } catch (error) {
    console.error('Failed to get settings:', error);
    return null;
  }
}

async function updateSettings(settings) {
  try {
    return await invoke('update_settings', { settings });
  } catch (error) {
    console.error('Failed to update settings:', error);
    showToast(t('toasts.settings_save_failed'), 'error');
    throw error;
  }
}

async function getAllHotkeys() {
  try {
    return await invoke('get_all_hotkeys');
  } catch (error) {
    console.error('Failed to get hotkeys:', error);
    return [];
  }
}

async function createHotkey(entryId, accelerator, scope) {
  try {
    const hotkey = await invoke('create_hotkey', { entryId, accelerator, scope });
    showToast(t('toasts.hotkey_created'), 'success');
    return hotkey;
  } catch (error) {
    console.error('Failed to create hotkey:', error);
    showToast(t('toasts.hotkey_create_failed', { error: error.message }), 'error');
    throw error;
  }
}

async function deleteHotkey(id) {
  try {
    await invoke('delete_hotkey', { id });
    showToast(t('toasts.hotkey_deleted'), 'success');
  } catch (error) {
    console.error('Failed to delete hotkey:', error);
    showToast(t('toasts.hotkey_delete_failed'), 'error');
    throw error;
  }
}

async function updateHotkey(id, accelerator, scope, enabled) {
  try {
    const hotkey = await invoke('update_hotkey', {
      id,
      accelerator,
      scope,
      enabled,
    });
    showToast(t('toasts.hotkey_updated'), 'success');
    return hotkey;
  } catch (error) {
    console.error('Failed to update hotkey:', error);
    showToast(t('toasts.hotkey_update_failed', { error: error.message }), 'error');
    throw error;
  }
}

function getEntryHotkeys(entryId) {
  return state.hotkeys.filter(hotkey => hotkey.entry_id === entryId && hotkey.scope === 'global');
}

function getPreferredEntryHotkey(entryId) {
  const matches = getEntryHotkeys(entryId);
  const enabled = matches.find(hotkey => hotkey.enabled);
  return enabled || matches[0] || null;
}

async function syncEntryHotkey(entryId, accelerator) {
  const trimmed = accelerator.trim();
  const matches = getEntryHotkeys(entryId);
  const primary = getPreferredEntryHotkey(entryId);

  if (!trimmed) {
    for (const hotkey of matches) {
      await deleteHotkey(hotkey.id);
    }
    await loadHotkeys();
    return;
  }

  if (!primary) {
    await createHotkey(entryId, trimmed, 'global');
    await loadHotkeys();
    return;
  }

  if (primary.accelerator !== trimmed || !primary.enabled || primary.scope !== 'global') {
    await updateHotkey(primary.id, trimmed, 'global', true);
  }

  const extras = matches.filter(hotkey => hotkey.id !== primary.id);
  for (const hotkey of extras) {
    await deleteHotkey(hotkey.id);
  }

  await loadHotkeys();
}

async function getAllTemplates() {
  try {
    return await invoke('get_all_templates');
  } catch (error) {
    console.error('Failed to get templates:', error);
    return [];
  }
}

async function createTemplate(input) {
  try {
    const template = await invoke('create_template', { input });
    showToast(t('toasts.template_created'), 'success');
    return template;
  } catch (error) {
    console.error('Failed to create template:', error);
    showToast(t('toasts.template_create_failed', { error: error.message }), 'error');
    throw error;
  }
}

async function updateTemplate(id, input) {
  try {
    const template = await invoke('update_template', { id, input });
    showToast(t('toasts.template_updated'), 'success');
    return template;
  } catch (error) {
    console.error('Failed to update template:', error);
    showToast(t('toasts.template_update_failed', { error: error.message }), 'error');
    throw error;
  }
}

async function deleteTemplate(id) {
  try {
    await invoke('delete_template', { id });
    showToast(t('toasts.template_deleted'), 'success');
  } catch (error) {
    console.error('Failed to delete template:', error);
    showToast(t('toasts.template_delete_failed'), 'error');
    throw error;
  }
}

async function renderTemplate(id, variables) {
  try {
    return await invoke('render_template', { id, variables });
  } catch (error) {
    console.error('Failed to render template:', error);
    return '';
  }
}

async function exportData() {
  try {
    const json = await invoke('export_data');
    const filePath = await invoke('save_file_dialog', { defaultName: 'opener-config.json' });
    if (filePath) {
      // Write file using Tauri FS API
      const { writeTextFile } = window.__TAURI__.fs;
      await writeTextFile(filePath, json);
      showToast(t('toasts.export_success'), 'success');
    }
  } catch (error) {
    console.error('Export failed:', error);
    showToast(t('toasts.export_failed', { error: error.message }), 'error');
  }
}

async function importData(strategy) {
  try {
    const filePath = await invoke('open_file_dialog');
    if (filePath) {
      const { readTextFile } = window.__TAURI__.fs;
      const jsonData = await readTextFile(filePath);
      const result = await invoke('import_data', { input: { json_data: jsonData, strategy } });
      showToast(t('toasts.import_complete', {
        entries: result.entries_imported,
        hotkeys: result.hotkeys_imported,
        templates: result.templates_imported,
      }), 'success');
      await loadAllData();
      applyLanguage(state.settings?.language || DEFAULT_LANGUAGE);
    }
  } catch (error) {
    console.error('Import failed:', error);
    showToast(t('toasts.import_failed', { error: error.message }), 'error');
  }
}

// ==================== UI Rendering ====================

function renderSearchResults(results, options = {}) {
  const container = document.getElementById('results-list');
  const emptyState = document.getElementById('empty-state');
  const showEmptyState = options.showEmptyState !== false;

  if (results.length === 0) {
    container.innerHTML = '';
    if (showEmptyState) {
      emptyState.classList.remove('hidden');
    } else {
      emptyState.classList.add('hidden');
    }
    return;
  }

  emptyState.classList.add('hidden');

  container.innerHTML = results.map((result, index) => {
    const entry = result.entry;
    const isSelected = index === state.selectedIndex;
    const iconSize = state.settings?.icon_size || 24;

    return `
      <div class="entry-item ${isSelected ? 'selected' : ''}"
           data-id="${entry.id}"
           data-index="${index}"
           tabindex="0">
        <div class="flex-shrink-0" style="width: ${iconSize}px; height: ${iconSize}px;">
          ${entry.icon_path ?
            `<img src="${escapeHtml(entry.icon_path)}" class="w-full h-full object-contain" onerror="this.style.display='none'">` :
            `<div class="w-full h-full rounded bg-gray-700 flex items-center justify-center text-gray-400">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
              </svg>
            </div>`
          }
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="font-medium truncate">${escapeHtml(entry.name)}</span>
            ${state.settings?.show_type_label !== false ?
              `<span class="type-badge ${entry.type}">${getEntryTypeLabel(entry.type)}</span>` : ''}
          </div>
          ${state.settings?.show_path !== false && entry.target ?
            `<div class="text-sm text-gray-500 truncate">${escapeHtml(entry.target)}</div>` : ''}
          ${state.settings?.show_description !== false && entry.description ?
            `<div class="text-sm text-gray-400 truncate">${escapeHtml(entry.description)}</div>` : ''}
        </div>
        <button class="btn-ghost p-1.5 rounded opacity-0 group-hover:opacity-100 entry-menu-btn" data-id="${entry.id}">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01"/>
          </svg>
        </button>
      </div>
    `;
  }).join('');

  // Add click handlers
  container.querySelectorAll('.entry-item').forEach(item => {
    item.addEventListener('click', (e) => {
      if (!e.target.closest('.entry-menu-btn')) {
        const id = item.dataset.id;
        handleEntryExecution(id);
      }
    });

    item.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      const id = item.dataset.id;
      showEntryContextMenu(e, id);
    });
  });
}

function getFilteredEntries(entries) {
  const query = state.entriesSearchQuery.trim().toLowerCase();
  if (!query) {
    return entries;
  }

  return entries.filter(entry => {
    const haystack = [
      entry.name,
      entry.target,
      entry.description,
      entry.tags,
      entry.type,
      entry.hotkey_filter,
      entry.hotkey_position,
    ].filter(Boolean).join(' ').toLowerCase();
    return haystack.includes(query);
  });
}

function pruneSelectedEntries(entries) {
  const validIds = new Set(entries.map(entry => entry.id));
  state.selectedEntryIds.forEach(id => {
    if (!validIds.has(id)) {
      state.selectedEntryIds.delete(id);
    }
  });
}

function updateEntriesSelectionUI(filteredEntries) {
  const selectAll = document.getElementById('entries-select-all');
  const deleteBtn = document.getElementById('btn-delete-selected-entries');
  const countEl = document.getElementById('entries-selected-count');

  const totalVisible = filteredEntries.length;
  const selectedVisible = filteredEntries.filter(entry => state.selectedEntryIds.has(entry.id)).length;
  const totalSelected = state.selectedEntryIds.size;

  if (selectAll) {
    selectAll.checked = totalVisible > 0 && selectedVisible === totalVisible;
    selectAll.indeterminate = selectedVisible > 0 && selectedVisible < totalVisible;
    selectAll.disabled = totalVisible === 0;
  }

  if (deleteBtn) {
    deleteBtn.disabled = totalSelected === 0;
  }

  if (countEl) {
    countEl.textContent = totalSelected > 0 ? t('settings.entries.selected_count', { count: totalSelected }) : '';
  }
}

function setEntryRowSelected(entryId, isSelected) {
  const row = document.querySelector(`.entry-row[data-id="${entryId}"]`);
  if (!row) return;

  row.classList.toggle('ring-1', isSelected);
  row.classList.toggle('ring-primary-500/40', isSelected);
  row.classList.toggle('bg-gray-800/80', isSelected);

  const checkbox = row.querySelector('.entry-select');
  if (checkbox) {
    checkbox.checked = isSelected;
  }
}

function toggleEntrySelection(entryId, isSelected) {
  const nextSelected = isSelected !== undefined ? isSelected : !state.selectedEntryIds.has(entryId);
  if (nextSelected) {
    state.selectedEntryIds.add(entryId);
  } else {
    state.selectedEntryIds.delete(entryId);
  }

  setEntryRowSelected(entryId, nextSelected);
  updateEntriesSelectionUI(getFilteredEntries(state.entries));
}

function handleSelectAllEntries(isSelected) {
  const filteredEntries = getFilteredEntries(state.entries);
  if (isSelected) {
    filteredEntries.forEach(entry => state.selectedEntryIds.add(entry.id));
  } else {
    filteredEntries.forEach(entry => state.selectedEntryIds.delete(entry.id));
  }
  renderEntriesList(state.entries);
}

async function handleDeleteSelectedEntries() {
  const selectedIds = Array.from(state.selectedEntryIds);
  if (selectedIds.length === 0) return;

  showConfirmModal(
    t('confirm.delete_entries_title'),
    t('confirm.delete_entries_message', { count: selectedIds.length }),
    t('confirm.irreversible'),
    async () => {
      let failedCount = 0;
      for (const id of selectedIds) {
        try {
          await deleteEntry(id, { silent: true });
        } catch (error) {
          failedCount += 1;
        }
      }

      await loadAllData();
      state.selectedEntryIds.clear();
      renderEntriesList(state.entries);

      if (failedCount === 0) {
        showToast(t('toasts.entries_deleted', { count: selectedIds.length }), 'success');
      } else {
        showToast(t('toasts.entries_delete_failed', { count: failedCount }), 'error');
      }
    }
  );
}

function renderEntriesList(entries) {
  const container = document.getElementById('entries-list');
  pruneSelectedEntries(entries);
  const filteredEntries = getFilteredEntries(entries);

  if (entries.length === 0) {
    container.innerHTML = `
      <div class="text-center py-8 text-gray-500">
        <p>${t('settings.entries.empty')}</p>
      </div>
    `;
    updateEntriesSelectionUI(filteredEntries);
    return;
  }

  if (filteredEntries.length === 0) {
    container.innerHTML = `
      <div class="text-center py-8 text-gray-500">
        <p>${t('settings.entries.no_match')}</p>
      </div>
    `;
    updateEntriesSelectionUI(filteredEntries);
    return;
  }

  container.innerHTML = filteredEntries.map(entry => {
    const isSelected = state.selectedEntryIds.has(entry.id);
    const selectedClass = isSelected ? 'ring-1 ring-primary-500/40 bg-gray-800/80' : '';
    return `
    <div class="entry-row card p-3 flex items-center gap-3 ${selectedClass}" data-id="${entry.id}">
      <label class="flex items-center">
        <input type="checkbox" class="entry-select w-4 h-4 rounded border-gray-600 text-primary-500 focus:ring-primary-500" data-id="${entry.id}" ${isSelected ? 'checked' : ''} aria-label="${escapeHtml(t('settings.entries.select_entry', { name: entry.name }))}">
      </label>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="font-medium">${escapeHtml(entry.name)}</span>
          <span class="type-badge ${entry.type}">${getEntryTypeLabel(entry.type)}</span>
          ${!entry.enabled ? `<span class="text-xs text-gray-500">${t('labels.disabled')}</span>` : ''}
        </div>
        <div class="text-sm text-gray-500 truncate">${escapeHtml(entry.target)}</div>
      </div>
      <div class="flex items-center gap-1">
        <button class="btn-ghost p-1.5 rounded text-emerald-400 hover:text-emerald-300" data-entry-action="test" title="${t('actions.test')}">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3l14 9-14 9V3z"/>
          </svg>
        </button>
        <button class="btn-ghost p-1.5 rounded" data-entry-action="edit" title="${t('actions.edit')}">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
          </svg>
        </button>
        <button class="btn-ghost p-1.5 rounded text-sky-400 hover:text-sky-300" data-entry-action="duplicate" title="${t('actions.duplicate')}">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"/>
          </svg>
        </button>
        <button class="btn-ghost p-1.5 rounded text-red-400 hover:text-red-300" data-entry-action="delete" title="${t('actions.delete')}">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
          </svg>
        </button>
      </div>
    </div>
  `;
  }).join('');

  container.querySelectorAll('.entry-select').forEach(checkbox => {
    checkbox.addEventListener('change', (e) => {
      e.stopPropagation();
      const entryId = checkbox.dataset.id;
      toggleEntrySelection(entryId, checkbox.checked);
    });
  });

  container.querySelectorAll('[data-entry-action]').forEach(button => {
    button.addEventListener('click', (e) => {
      e.stopPropagation();
      const action = button.dataset.entryAction;
      const entryId = button.closest('.entry-row')?.dataset.id;
      if (!entryId) return;

      if (action === 'test') {
        testEntry(entryId);
      } else if (action === 'edit') {
        editEntry(entryId);
      } else if (action === 'duplicate') {
        duplicateEntry(entryId);
      } else if (action === 'delete') {
        confirmDeleteEntry(entryId);
      }
    });
  });

  container.querySelectorAll('.entry-row').forEach(row => {
    row.addEventListener('click', (e) => {
      if (e.target.closest('button') || e.target.closest('input') || e.target.closest('label')) {
        return;
      }
      const entryId = row.dataset.id;
      toggleEntrySelection(entryId);
    });
  });

  updateEntriesSelectionUI(filteredEntries);
}

function renderTemplatesList(templates) {
  const container = document.getElementById('templates-list');
  if (!container) {
    return;
  }

  if (templates.length === 0) {
    container.innerHTML = `
      <div class="text-center py-8 text-gray-500">
        <p>${t('settings.templates.empty')}</p>
      </div>
    `;
    return;
  }

  container.innerHTML = templates.map(template => `
    <div class="card p-3" data-template-id="${template.id}">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <span class="font-medium">${escapeHtml(template.name)}</span>
          <span class="type-badge script">${getTemplateLanguageLabel(template.language)}</span>
        </div>
        <div class="flex items-center gap-1">
          <button class="btn-ghost p-1.5 rounded text-green-400" data-template-action="use" title="${t('actions.use_template')}">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
          </button>
          <button class="btn-ghost p-1.5 rounded" data-template-action="edit" title="${t('actions.edit')}">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
            </svg>
          </button>
          <button class="btn-ghost p-1.5 rounded text-red-400 hover:text-red-300" data-template-action="delete" title="${t('actions.delete')}">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
            </svg>
          </button>
        </div>
      </div>
      ${template.description ? `<p class="text-sm text-gray-400 mb-2">${escapeHtml(template.description)}</p>` : ''}
      <pre class="text-xs bg-gray-900 p-2 rounded overflow-x-auto">${escapeHtml(template.template_content)}</pre>
      ${template.variables.length > 0 ? `
        <div class="mt-2 flex flex-wrap gap-1">
          ${template.variables.map(v => `<span class="text-xs bg-gray-700 px-2 py-0.5 rounded">${escapeHtml(v.name)}</span>`).join('')}
        </div>
      ` : ''}
    </div>
  `).join('');

  container.querySelectorAll('[data-template-action]').forEach(button => {
    button.addEventListener('click', (e) => {
      e.stopPropagation();
      const action = button.dataset.templateAction;
      const templateId = button.closest('[data-template-id]')?.dataset.templateId;
      if (!templateId) return;

      if (action === 'use') {
        useTemplate(templateId);
      } else if (action === 'edit') {
        editTemplate(templateId);
      } else if (action === 'delete') {
        confirmDeleteTemplate(templateId);
      }
    });
  });
}

function updateSettingsUI(settings) {
  document.getElementById('setting-icon-size').value = settings.icon_size;
  document.getElementById('setting-sort-strategy').value = settings.sort_strategy;
  document.getElementById('setting-max-results').value = settings.max_results;
  document.getElementById('setting-show-path').checked = settings.show_path;
  document.getElementById('setting-show-type').checked = settings.show_type_label;
  document.getElementById('setting-show-desc').checked = settings.show_description;
  document.getElementById('setting-confirm-dangerous').checked = settings.confirm_dangerous_commands;
  document.getElementById('setting-auto-launch').checked = settings.auto_launch === true;
  document.getElementById('setting-app-hotkey').value = settings.app_hotkey || '';
  document.getElementById('setting-language').value = normalizeLanguage(settings.language || DEFAULT_LANGUAGE);
}

// ==================== Event Handlers ====================

async function handleSearch(query) {
  state.selectedIndex = 0;

  if (query.trim() === '') {
    state.searchResults = [];
    renderSearchResults([], { showEmptyState: false });
  } else {
    const results = await searchEntries(query);
    state.searchResults = results;
    renderSearchResults(results);
  }
}

function handleEntriesSearchInput(e) {
  state.entriesSearchQuery = e.target.value;
  renderEntriesList(state.entries);
}

function clearSearchAfterExecution() {
  const searchInput = document.getElementById('search-input');
  if (!searchInput) return;
  if (searchInput.value) {
    searchInput.value = '';
  }
  void handleSearch('');
}

function hideAppWindow() {
  invoke('minimize_window').catch((error) => {
    console.error('Failed to hide window:', error);
  });
}

function fireAndForgetExecution(id) {
  executeEntry(id, { silent: true }).catch(() => {});
}

async function handleEntryExecution(id, options = {}) {
  const entry = state.entries.find(e => e.id === id);
  if (!entry) return;
  const shouldClearSearch = options.clearSearch !== false;
  const shouldHideWindow = options.hideWindow === true;
  const shouldExecuteImmediately = options.immediate === true;

  const dangerousTypes = ['cmd', 'wsl', 'ssh', 'script', 'ahk', 'hotkey_app'];
  const needsConfirmation = state.settings?.confirm_dangerous_commands &&
                            dangerousTypes.includes(entry.type) &&
                            entry.confirm_before_run !== false;

  const applyExecutionSideEffects = () => {
    if (shouldClearSearch) {
      clearSearchAfterExecution();
    }
    if (shouldHideWindow) {
      hideAppWindow();
    }
  };

  if (needsConfirmation) {
    showConfirmModal(
      t('confirm.execute_command_title'),
      t('confirm.execute_command_message', { type: getEntryTypeLabel(entry.type) }),
      entry.target,
      async () => {
        if (shouldExecuteImmediately) {
          applyExecutionSideEffects();
          fireAndForgetExecution(id);
          return;
        }
        await executeEntry(id);
        applyExecutionSideEffects();
      }
    );
  } else {
    if (shouldExecuteImmediately) {
      applyExecutionSideEffects();
      fireAndForgetExecution(id);
      return;
    }
    await executeEntry(id);
    applyExecutionSideEffects();
  }
}

function testEntry(id) {
  handleEntryExecution(id, { clearSearch: false });
}

function handleKeyboardNavigation(e) {
  const searchInput = document.getElementById('search-input');
  const isSearchFocused = document.activeElement === searchInput;

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      if (state.searchResults.length > 0) {
        state.selectedIndex = Math.min(state.selectedIndex + 1, state.searchResults.length - 1);
        renderSearchResults(state.searchResults);
        scrollToSelected();
      }
      break;

    case 'ArrowUp':
      e.preventDefault();
      if (state.searchResults.length > 0) {
        state.selectedIndex = Math.max(state.selectedIndex - 1, 0);
        renderSearchResults(state.searchResults);
        scrollToSelected();
      }
      break;

    case 'Enter':
      if (state.searchResults.length > 0 && state.selectedIndex >= 0) {
        const selectedEntry = state.searchResults[state.selectedIndex].entry;
        handleEntryExecution(selectedEntry.id, { clearSearch: true, hideWindow: true, immediate: true });
      }
      break;

    case 'Escape':
      if (isSearchFocused && searchInput.value) {
        searchInput.value = '';
        handleSearch('');
      } else {
        closeAllModals();
      }
      break;
  }
}

function focusSearchInput() {
  const searchInput = document.getElementById('search-input');
  if (!searchInput) return;
  if (document.activeElement === searchInput) return;
  searchInput.focus();
}

function shouldFocusSearchOnClick(target) {
  if (!target) return false;
  if (target.closest('.modal')) return false;
  if (target.closest('input, textarea, select, button, a, [contenteditable="true"]')) return false;
  return true;
}

function scrollToSelected() {
  const selected = document.querySelector('.entry-item.selected');
  if (selected) {
    selected.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
}

// ==================== Modal Handlers ====================

function showSettingsModal() {
  document.getElementById('settings-modal').classList.remove('hidden');
  switchSettingsTab('entries');
}

function closeSettingsModal() {
  document.getElementById('settings-modal').classList.add('hidden');
}

function showAboutModal() {
  closeAllModals();
  document.getElementById('about-modal').classList.remove('hidden');
}

function closeAboutModal() {
  document.getElementById('about-modal').classList.add('hidden');
}

function showHelpModal() {
  closeAllModals();
  document.getElementById('help-modal').classList.remove('hidden');
}

function closeHelpModal() {
  document.getElementById('help-modal').classList.add('hidden');
}

function switchSettingsTab(tabName) {
  state.currentTab = tabName;

  // Update sidebar
  document.querySelectorAll('.sidebar-item').forEach(item => {
    item.classList.toggle('active', item.dataset.tab === tabName);
  });

  // Update content
  document.querySelectorAll('.tab-content').forEach(content => {
    content.classList.add('hidden');
  });
  document.getElementById(`tab-${tabName}`).classList.remove('hidden');

  // Refresh content
    if (tabName === 'entries') {
      renderEntriesList(state.entries);
    } else if (tabName === 'templates') {
      renderTemplatesList(state.templates);
    }
  }

function showAddEntryModal() {
  document.getElementById('entry-modal-title').textContent = t('entry.modal_add');
  document.getElementById('entry-form').reset();
  document.getElementById('entry-id').value = '';
  document.getElementById('entry-hotkey').value = '';
  const hotkeyPositionInput = document.getElementById('entry-hotkey-position');
  if (hotkeyPositionInput) hotkeyPositionInput.dataset.touched = '';
  const detectHiddenInput = document.getElementById('entry-hotkey-detect-hidden');
  if (detectHiddenInput) detectHiddenInput.dataset.touched = '';
  updateEntryFormFields();
  document.getElementById('entry-modal').classList.remove('hidden');
}

function showEditEntryModal(entry) {
  document.getElementById('entry-modal-title').textContent = t('entry.modal_edit');
  document.getElementById('entry-id').value = entry.id;
  document.getElementById('entry-name').value = entry.name;
  document.getElementById('entry-type').value = entry.type;
  const targetInput = document.getElementById('entry-target');
  const argsInput = document.getElementById('entry-args');
  if (entry.type === 'hotkey_app') {
    const { executable, args } = splitHotkeyTarget(entry.target || '');
    targetInput.value = executable || entry.target || '';
    argsInput.value = args || entry.args || '';
  } else {
    targetInput.value = entry.target;
    argsInput.value = entry.args || '';
  }
  document.getElementById('entry-workdir').value = entry.workdir || '';
  document.getElementById('entry-description').value = entry.description || '';
  document.getElementById('entry-tags').value = entry.tags || '';
  const entryHotkey = getPreferredEntryHotkey(entry.id);
  document.getElementById('entry-hotkey').value = entryHotkey ? entryHotkey.accelerator : '';
  document.getElementById('entry-icon').value = entry.icon_path || '';
  document.getElementById('entry-show-terminal').checked = entry.show_terminal || false;
  document.getElementById('entry-confirm').checked = entry.confirm_before_run || false;

  // Hotkey app fields
  document.getElementById('entry-hotkey-filter').value = entry.hotkey_filter || '';
  const hotkeyPositionInput = document.getElementById('entry-hotkey-position');
  if (hotkeyPositionInput) {
    hotkeyPositionInput.value = entry.hotkey_position || 'keep';
    hotkeyPositionInput.dataset.touched = 'true';
  }
  const detectHiddenInput = document.getElementById('entry-hotkey-detect-hidden');
  if (detectHiddenInput) {
    detectHiddenInput.checked = entry.hotkey_detect_hidden !== false;
    detectHiddenInput.dataset.touched = 'true';
  }

  // SSH fields
  document.getElementById('entry-ssh-host').value = entry.ssh_host || '';
  document.getElementById('entry-ssh-user').value = entry.ssh_user || '';
  document.getElementById('entry-ssh-port').value = entry.ssh_port || 22;

  // WSL fields
  document.getElementById('entry-wsl-distro').value = entry.wsl_distro || '';

  // Script content
  if (entry.type === 'script' || entry.type === 'ahk') {
    document.getElementById('entry-script-content').value = entry.target;
  }

  updateEntryFormFields();
  document.getElementById('entry-modal').classList.remove('hidden');
}

function closeEntryModal() {
  document.getElementById('entry-modal').classList.add('hidden');
}

function syncEntryFormRequirements() {
  const type = document.getElementById('entry-type').value;
  const targetInput = document.getElementById('entry-target');
  const workdirInput = document.getElementById('entry-workdir');
  const scriptContentInput = document.getElementById('entry-script-content');
  const sshHostInput = document.getElementById('entry-ssh-host');
  const sshUserInput = document.getElementById('entry-ssh-user');
  const sshPortInput = document.getElementById('entry-ssh-port');
  const hotkeyInput = document.getElementById('entry-hotkey');
  const hotkeyFilterInput = document.getElementById('entry-hotkey-filter');
  const hotkeyPositionInput = document.getElementById('entry-hotkey-position');
  const hotkeyDetectHiddenInput = document.getElementById('entry-hotkey-detect-hidden');

  const isSsh = type === 'ssh';
  const isScript = type === 'script' || type === 'ahk';
  const isHotkeyApp = type === 'hotkey_app';

  targetInput.disabled = isSsh;
  targetInput.required = !isSsh && !isScript;
  workdirInput.required = isHotkeyApp;
  hotkeyInput.required = isHotkeyApp;

  sshHostInput.disabled = !isSsh;
  sshHostInput.required = isSsh;
  sshUserInput.disabled = !isSsh;
  sshPortInput.disabled = !isSsh;

  scriptContentInput.disabled = !isScript;
  if (isScript) {
    const hasTarget = targetInput.value.trim().length > 0;
    const hasScript = scriptContentInput.value.trim().length > 0;
    targetInput.required = !hasScript;
    scriptContentInput.required = !hasTarget;
  } else {
    scriptContentInput.required = false;
  }

  hotkeyFilterInput.disabled = !isHotkeyApp;
  hotkeyFilterInput.required = isHotkeyApp;
  hotkeyPositionInput.disabled = !isHotkeyApp;
  hotkeyPositionInput.required = isHotkeyApp;
  hotkeyDetectHiddenInput.disabled = !isHotkeyApp;
}

function updateEntryFormFields() {
  const type = document.getElementById('entry-type').value;

  // Show/hide fields based on type
  const sshFields = document.getElementById('field-ssh');
  const wslFields = document.getElementById('field-wsl');
  const scriptFields = document.getElementById('field-script');
  const hotkeyAppFields = document.getElementById('field-hotkey-app');
  const targetField = document.getElementById('field-target');
  const argsField = document.getElementById('field-args');

  sshFields.classList.toggle('hidden', type !== 'ssh');
  wslFields.classList.toggle('hidden', type !== 'wsl');
  scriptFields.classList.toggle('hidden', type !== 'script' && type !== 'ahk');
  hotkeyAppFields.classList.toggle('hidden', type !== 'hotkey_app');
  targetField.classList.toggle('hidden', type === 'ssh');
  argsField.classList.toggle('hidden', type === 'url' || type === 'ssh');

  // Update target label and placeholder
  const targetLabel = targetField.querySelector('label');
  const targetInput = document.getElementById('entry-target');

  switch (type) {
    case 'url':
      targetLabel.textContent = t('entry.target_labels.url');
      targetInput.placeholder = t('entry.target_placeholders.url');
      break;
    case 'app':
    case 'file':
    case 'shortcut':
      targetLabel.textContent = t('entry.target_labels.file');
      targetInput.placeholder = t('entry.target_placeholders.file');
      break;
    case 'dir':
      targetLabel.textContent = t('entry.target_labels.dir');
      targetInput.placeholder = t('entry.target_placeholders.dir');
      break;
    case 'cmd':
      targetLabel.textContent = t('entry.target_labels.command');
      targetInput.placeholder = t('entry.target_placeholders.command');
      break;
    case 'wsl':
      targetLabel.textContent = t('entry.target_labels.wsl_command');
      targetInput.placeholder = t('entry.target_placeholders.wsl_command');
      break;
    case 'script':
    case 'ahk':
      targetLabel.textContent = t('entry.target_labels.script_path');
      targetInput.placeholder = t('entry.target_placeholders.script_path');
      break;
    case 'hotkey_app':
      targetLabel.textContent = t('entry.target_labels.executable');
      targetInput.placeholder = t('entry.target_placeholders.executable');
      break;
    default:
      targetLabel.textContent = t('entry.target_labels.target');
      targetInput.placeholder = '';
  }

  const nameLabel = document.querySelector('#entry-name')?.closest('div')?.querySelector('label');
  const hotkeyLabel = document.querySelector('#field-hotkey .label');
  const hotkeyHelper = document.querySelector('#field-hotkey p');

  if (type === 'hotkey_app') {
    if (nameLabel) nameLabel.textContent = t('entry.fields.app_name');
    if (hotkeyLabel) hotkeyLabel.textContent = t('entry.fields.hotkey');
    if (hotkeyHelper) hotkeyHelper.textContent = t('entry.helpers.hotkey_required');

    const hotkeyPositionInput = document.getElementById('entry-hotkey-position');
    if (hotkeyPositionInput && hotkeyPositionInput.dataset.touched !== 'true') {
      hotkeyPositionInput.value = 'keep';
    }
    const detectHiddenInput = document.getElementById('entry-hotkey-detect-hidden');
    if (detectHiddenInput && detectHiddenInput.dataset.touched !== 'true') {
      detectHiddenInput.checked = true;
    }
  } else {
    if (nameLabel) nameLabel.textContent = t('entry.fields.name');
    if (hotkeyLabel) hotkeyLabel.textContent = t('entry.fields.global_hotkey');
    if (hotkeyHelper) hotkeyHelper.textContent = t('entry.helpers.global_hotkey');
  }

  syncEntryFormRequirements();
}

function buildSshTarget(host, user, port) {
  if (!host) return '';
  const userPrefix = user ? `${user}@` : '';
  const portSuffix = port && port !== 22 ? `:${port}` : '';
  return `${userPrefix}${host}${portSuffix}`;
}

function splitHotkeyTarget(value) {
  const trimmed = value.trim();
  if (!trimmed) {
    return { executable: '', args: '' };
  }

  const firstChar = trimmed[0];
  if (firstChar === '"' || firstChar === '\'') {
    const endIndex = trimmed.indexOf(firstChar, 1);
    if (endIndex > 0) {
      const executable = trimmed.slice(1, endIndex);
      const args = trimmed.slice(endIndex + 1).trim();
      return { executable, args };
    }
  }

  const lower = trimmed.toLowerCase();
  const exeIndex = lower.lastIndexOf('.exe');
  if (exeIndex !== -1) {
    const endIndex = exeIndex + 4;
    const executable = trimmed.slice(0, endIndex).trim();
    const args = trimmed.slice(endIndex).trim();
    return { executable, args };
  }

  const firstSpace = trimmed.search(/\s/);
  if (firstSpace === -1) {
    return { executable: trimmed, args: '' };
  }

  return {
    executable: trimmed.slice(0, firstSpace).trim(),
    args: trimmed.slice(firstSpace).trim(),
  };
}

function buildEntryInputFromForm() {
  const type = document.getElementById('entry-type').value;
  const targetInput = document.getElementById('entry-target');
  let target = targetInput.value;

  if ((type === 'script' || type === 'ahk') && !target.trim()) {
    const scriptContent = document.getElementById('entry-script-content').value;
    if (scriptContent.trim()) {
      target = scriptContent;
    }
  }

  let sshHost = '';
  let sshUser = '';
  let sshPort = 22;

  if (type === 'ssh') {
    sshHost = document.getElementById('entry-ssh-host').value.trim();
    sshUser = document.getElementById('entry-ssh-user').value.trim();
    const portValue = document.getElementById('entry-ssh-port').value;
    const parsedPort = Number.parseInt(portValue, 10);
    sshPort = Number.isNaN(parsedPort) || parsedPort <= 0 ? 22 : parsedPort;
    target = buildSshTarget(sshHost, sshUser, sshPort);
  }

  const rawArgsValue = document.getElementById('entry-args').value;
  const trimmedArgsValue = rawArgsValue.trim();
  let argsValue = rawArgsValue;

  if (type === 'hotkey_app') {
    const executable = targetInput.value.trim();
    target = trimmedArgsValue ? `${executable} ${trimmedArgsValue}` : executable;
    argsValue = '';
  }

  const input = {
    name: document.getElementById('entry-name').value,
    type: type,
    target: target,
    args: argsValue || null,
    workdir: document.getElementById('entry-workdir').value || null,
    description: document.getElementById('entry-description').value || null,
    tags: document.getElementById('entry-tags').value || null,
    icon_path: document.getElementById('entry-icon').value || null,
    show_terminal: document.getElementById('entry-show-terminal').checked,
    confirm_before_run: document.getElementById('entry-confirm').checked,
  };
  const hotkeyValue = document.getElementById('entry-hotkey').value.trim();

  if (type === 'ssh') {
    input.ssh_host = sshHost || null;
    input.ssh_user = sshUser || null;
    input.ssh_port = sshPort;
  }

  if (type === 'wsl') {
    input.wsl_distro = document.getElementById('entry-wsl-distro').value || null;
  }

  if (type === 'hotkey_app') {
    input.hotkey_filter = document.getElementById('entry-hotkey-filter').value || null;
    input.hotkey_position = document.getElementById('entry-hotkey-position').value || 'keep';
    input.hotkey_detect_hidden = document.getElementById('entry-hotkey-detect-hidden').checked;
  }

  return { input, hotkeyValue };
}

async function handleEntryFormSubmit(e) {
  e.preventDefault();

  const id = document.getElementById('entry-id').value;
  const { input, hotkeyValue } = buildEntryInputFromForm();

  try {
    let savedEntry;
    if (id) {
      savedEntry = await updateEntry(id, input);
    } else {
      savedEntry = await createEntry(input);
      document.getElementById('entry-id').value = savedEntry.id;
      document.getElementById('entry-modal-title').textContent = t('entry.modal_edit');
    }

    try {
      await syncEntryHotkey(savedEntry.id, hotkeyValue);
    } catch (error) {
      await loadAllData();
      renderEntriesList(state.entries);
      return;
    }

    await loadAllData();
    closeEntryModal();
    renderEntriesList(state.entries);
  } catch (error) {
    // Error already shown via toast
  }
}

async function handleEntryFormTest() {
  syncEntryFormRequirements();
  const form = document.getElementById('entry-form');
  if (!form.reportValidity()) {
    return;
  }

  const { input } = buildEntryInputFromForm();
  if (!input) return;

  const dangerousTypes = ['cmd', 'wsl', 'ssh', 'script', 'ahk', 'hotkey_app'];
  const needsConfirmation = state.settings?.confirm_dangerous_commands &&
                            dangerousTypes.includes(input.type) &&
                            input.confirm_before_run !== false;

  const runTest = async () => {
    try {
      await executeEntryInput(input);
    } catch (error) {
      // Error already shown via toast.
    }
  };

  if (needsConfirmation) {
    showConfirmModal(
      t('confirm.execute_command_title'),
      t('confirm.execute_command_message', { type: getEntryTypeLabel(input.type) }),
      input.target,
      async () => {
        await runTest();
      }
    );
  } else {
    await runTest();
  }
}

function showConfirmModal(title, message, details, onConfirm) {
  document.getElementById('confirm-title').textContent = title;
  document.getElementById('confirm-message').textContent = message;
  document.getElementById('confirm-details').textContent = details;

  const confirmBtn = document.getElementById('btn-confirm-ok');
  const newConfirmBtn = confirmBtn.cloneNode(true);
  confirmBtn.parentNode.replaceChild(newConfirmBtn, confirmBtn);

  newConfirmBtn.addEventListener('click', async () => {
    await onConfirm();
    closeConfirmModal();
  });

  document.getElementById('confirm-modal').classList.remove('hidden');
}

function closeConfirmModal() {
  document.getElementById('confirm-modal').classList.add('hidden');
}

function showAddTemplateModal() {
  document.getElementById('template-modal-title').textContent = t('template.modal_add');
  document.getElementById('template-form').reset();
  document.getElementById('template-id').value = '';
  document.getElementById('template-variables').innerHTML = '';
  document.getElementById('template-modal').classList.remove('hidden');
}

function showEditTemplateModal(template) {
  document.getElementById('template-modal-title').textContent = t('template.modal_edit');
  document.getElementById('template-id').value = template.id;
  document.getElementById('template-name').value = template.name;
  document.getElementById('template-description').value = template.description || '';
  document.getElementById('template-language').value = template.language;
  document.getElementById('template-content').value = template.template_content;

  // Render variables
  renderTemplateVariables(template.variables);

  document.getElementById('template-modal').classList.remove('hidden');
}

function closeTemplateModal() {
  document.getElementById('template-modal').classList.add('hidden');
}

function renderTemplateVariables(variables) {
  const container = document.getElementById('template-variables');

  container.innerHTML = variables.map((v, i) => `
    <div class="card p-3 space-y-2" data-index="${i}">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">${t('template.variable_label', { index: i + 1 })}</span>
        <button type="button" class="text-red-400 hover:text-red-300 text-sm" data-template-variable-action="remove">${t('template.remove_variable')}</button>
      </div>
      <div class="grid grid-cols-2 gap-2">
        <input type="text" class="input var-name" value="${escapeHtml(v.name)}" placeholder="${t('template.placeholders.var_name')}" required>
        <select class="input var-type">
          <option value="string" ${v.var_type === 'string' ? 'selected' : ''}>${t('template.variable_types.string')}</option>
          <option value="number" ${v.var_type === 'number' ? 'selected' : ''}>${t('template.variable_types.number')}</option>
          <option value="path" ${v.var_type === 'path' ? 'selected' : ''}>${t('template.variable_types.path')}</option>
          <option value="choice" ${v.var_type === 'choice' ? 'selected' : ''}>${t('template.variable_types.choice')}</option>
          <option value="boolean" ${v.var_type === 'boolean' ? 'selected' : ''}>${t('template.variable_types.boolean')}</option>
        </select>
      </div>
      <input type="text" class="input var-label" value="${escapeHtml(v.label)}" placeholder="${t('template.placeholders.var_label')}">
      <input type="text" class="input var-default" value="${escapeHtml(v.default_value || '')}" placeholder="${t('template.placeholders.var_default')}">
      <label class="flex items-center gap-2">
        <input type="checkbox" class="var-required" ${v.required ? 'checked' : ''}>
        <span class="text-sm">${t('template.required')}</span>
      </label>
    </div>
  `).join('');
}

function setupTemplateVariableActions() {
  const container = document.getElementById('template-variables');
  if (!container || container.dataset.actionsBound === 'true') return;

  container.dataset.actionsBound = 'true';
  container.addEventListener('click', (e) => {
    const actionButton = e.target.closest('[data-template-variable-action]');
    if (!actionButton) return;
    if (actionButton.dataset.templateVariableAction !== 'remove') return;

    const item = actionButton.closest('[data-index]');
    if (!item) return;
    removeTemplateVariable(item.dataset.index);
  });
}

function removeTemplateVariable(index) {
  const container = document.getElementById('template-variables');
  const item = container.querySelector(`[data-index="${index}"]`);
  if (item) item.remove();

  // Re-index remaining items
  container.querySelectorAll('[data-index]').forEach((el, i) => {
    el.dataset.index = i;
    el.querySelector('.text-sm.font-medium').textContent = t('template.variable_label', { index: i + 1 });
  });
};

function addTemplateVariable() {
  const container = document.getElementById('template-variables');
  const index = container.children.length;

  const html = `
    <div class="card p-3 space-y-2" data-index="${index}">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">${t('template.variable_label', { index: index + 1 })}</span>
        <button type="button" class="text-red-400 hover:text-red-300 text-sm" data-template-variable-action="remove">${t('template.remove_variable')}</button>
      </div>
      <div class="grid grid-cols-2 gap-2">
        <input type="text" class="input var-name" placeholder="${t('template.placeholders.var_name')}" required>
        <select class="input var-type">
          <option value="string">${t('template.variable_types.string')}</option>
          <option value="number">${t('template.variable_types.number')}</option>
          <option value="path">${t('template.variable_types.path')}</option>
          <option value="choice">${t('template.variable_types.choice')}</option>
          <option value="boolean">${t('template.variable_types.boolean')}</option>
        </select>
      </div>
      <input type="text" class="input var-label" placeholder="${t('template.placeholders.var_label')}">
      <input type="text" class="input var-default" placeholder="${t('template.placeholders.var_default')}">
      <label class="flex items-center gap-2">
        <input type="checkbox" class="var-required">
        <span class="text-sm">${t('template.required')}</span>
      </label>
    </div>
  `;

  container.insertAdjacentHTML('beforeend', html);
}

function getTemplateVariables() {
  const container = document.getElementById('template-variables');
  const variables = [];

  container.querySelectorAll('[data-index]').forEach(el => {
    variables.push({
      name: el.querySelector('.var-name').value,
      var_type: el.querySelector('.var-type').value,
      label: el.querySelector('.var-label').value,
      default_value: el.querySelector('.var-default').value || null,
      required: el.querySelector('.var-required').checked,
      choices: null,
      validation_regex: null,
    });
  });

  return variables;
}

async function handleTemplateFormSubmit(e) {
  e.preventDefault();

  const id = document.getElementById('template-id').value;

  const input = {
    name: document.getElementById('template-name').value,
    description: document.getElementById('template-description').value || null,
    language: document.getElementById('template-language').value,
    template_content: document.getElementById('template-content').value,
    variables: getTemplateVariables(),
  };

  try {
    if (id) {
      await updateTemplate(id, input);
    } else {
      await createTemplate(input);
    }

    await loadTemplates();
    closeTemplateModal();
    renderTemplatesList(state.templates);
  } catch (error) {
    // Error already shown via toast
  }
}

function showUseTemplateModal(template) {
  document.getElementById('use-template-title').textContent = t('template.modal_use_named', { name: template.name });
  document.getElementById('use-template-id').value = template.id;
  document.getElementById('use-template-entry-name').value = '';

  // Render variable inputs
  const container = document.getElementById('use-template-variables');
  container.innerHTML = template.variables.map(v => `
    <div>
      <label class="label">${escapeHtml(v.label || v.name)}${v.required ? ' *' : ''}</label>
      ${v.var_type === 'choice' && v.choices ? `
        <select class="input template-var" data-name="${v.name}" ${v.required ? 'required' : ''}>
          ${v.choices.map(c => `<option value="${c}" ${c === v.default_value ? 'selected' : ''}>${c}</option>`).join('')}
        </select>
      ` : v.var_type === 'boolean' ? `
        <label class="flex items-center gap-2">
          <input type="checkbox" class="template-var" data-name="${v.name}" ${v.default_value === 'true' ? 'checked' : ''}>
          <span>${escapeHtml(v.label || v.name)}</span>
        </label>
      ` : `
        <input type="${v.var_type === 'number' ? 'number' : 'text'}"
               class="input template-var"
               data-name="${v.name}"
               value="${escapeHtml(v.default_value || '')}"
               ${v.required ? 'required' : ''}>
      `}
    </div>
  `).join('');

  // Initial preview
  updateTemplatePreview(template);

  // Add event listeners for preview updates
  container.querySelectorAll('.template-var').forEach(input => {
    input.addEventListener('input', () => updateTemplatePreview(template));
    input.addEventListener('change', () => updateTemplatePreview(template));
  });

  document.getElementById('use-template-modal').classList.remove('hidden');
}

async function updateTemplatePreview(template) {
  const variables = {};
  document.querySelectorAll('#use-template-variables .template-var').forEach(input => {
    const name = input.dataset.name;
    if (input.type === 'checkbox') {
      variables[name] = input.checked ? 'true' : 'false';
    } else {
      variables[name] = input.value;
    }
  });

  const rendered = await renderTemplate(template.id, variables);
  document.getElementById('use-template-preview').textContent = rendered;
}

function closeUseTemplateModal() {
  document.getElementById('use-template-modal').classList.add('hidden');
}

async function handleUseTemplateFormSubmit(e) {
  e.preventDefault();

  const templateId = document.getElementById('use-template-id').value;
  const entryName = document.getElementById('use-template-entry-name').value;
  const template = state.templates.find(t => t.id === templateId);

  if (!template) return;

  const variables = {};
  document.querySelectorAll('#use-template-variables .template-var').forEach(input => {
    const name = input.dataset.name;
    if (input.type === 'checkbox') {
      variables[name] = input.checked ? 'true' : 'false';
    } else {
      variables[name] = input.value;
    }
  });

  const rendered = await renderTemplate(templateId, variables);

  // Determine entry type from template language
  let entryType = 'script';
  if (template.language === 'ssh') entryType = 'ssh';
  else if (template.language === 'wsl') entryType = 'wsl';
  else if (template.language === 'ahk') entryType = 'ahk';
  else if (template.language === 'cmd' || template.language === 'powershell') entryType = 'cmd';

  const input = {
    name: entryName,
    type: entryType,
    target: rendered,
    description: t('entry.created_from_template', { name: template.name }),
  };

  try {
    await createEntry(input);
    await loadAllData();
    closeUseTemplateModal();
    showToast(t('toasts.entry_from_template'), 'success');
  } catch (error) {
    // Error already shown via toast
  }
}

function closeAllModals() {
  document.getElementById('settings-modal').classList.add('hidden');
  document.getElementById('about-modal').classList.add('hidden');
  document.getElementById('help-modal').classList.add('hidden');
  document.getElementById('entry-modal').classList.add('hidden');
  document.getElementById('template-modal').classList.add('hidden');
  document.getElementById('use-template-modal').classList.add('hidden');
  document.getElementById('confirm-modal').classList.add('hidden');
}

// ==================== Shared Actions ====================

async function duplicateEntry(id) {
  const entry = state.entries.find(e => e.id === id);
  if (!entry) return;

  const input = {
    name: buildDuplicateName(entry.name),
    type: entry.type,
    target: entry.target,
    args: entry.args ?? null,
    workdir: entry.workdir ?? null,
    icon_path: entry.icon_path ?? null,
    tags: entry.tags ?? null,
    description: entry.description ?? null,
    enabled: entry.enabled,
    confirm_before_run: entry.confirm_before_run ?? null,
    show_terminal: entry.show_terminal ?? null,
    wsl_distro: entry.wsl_distro ?? null,
    ssh_host: entry.ssh_host ?? null,
    ssh_user: entry.ssh_user ?? null,
    ssh_port: entry.ssh_port ?? null,
    ssh_key_id: entry.ssh_key_id ?? null,
    env_vars: entry.env_vars ?? null,
    hotkey_filter: entry.hotkey_filter ?? null,
    hotkey_position: entry.hotkey_position ?? null,
    hotkey_detect_hidden: entry.hotkey_detect_hidden ?? null,
  };

  try {
    const newEntry = await createEntry(input);
    await loadAllData();
    renderEntriesList(state.entries);
    showEditEntryModal(newEntry);
  } catch (error) {
    // Error already shown via toast
  }
}

async function editEntry(id) {
  const entry = state.entries.find(e => e.id === id);
  if (entry) {
    showEditEntryModal(entry);
  }
}

function confirmDeleteEntry(id) {
  const entry = state.entries.find(e => e.id === id);
  if (entry) {
    showConfirmModal(
      t('confirm.delete_entry_title'),
      t('confirm.delete_entry_message', { name: entry.name }),
      t('confirm.irreversible'),
      async () => {
        await deleteEntry(id);
        await loadAllData();
        renderEntriesList(state.entries);
      }
    );
  }
}

async function editTemplate(id) {
  const template = state.templates.find(t => t.id === id);
  if (template) {
    showEditTemplateModal(template);
  }
}

function confirmDeleteTemplate(id) {
  const template = state.templates.find(t => t.id === id);
  if (template) {
    showConfirmModal(
      t('confirm.delete_template_title'),
      t('confirm.delete_template_message', { name: template.name }),
      t('confirm.irreversible'),
      async () => {
        await deleteTemplate(id);
        await loadTemplates();
        renderTemplatesList(state.templates);
      }
    );
  }
}

function useTemplate(id) {
  const template = state.templates.find(t => t.id === id);
  if (template) {
    showUseTemplateModal(template);
  }
}

window.editEntry = editEntry;
window.testEntry = testEntry;
window.confirmDeleteEntry = confirmDeleteEntry;
window.editTemplate = editTemplate;
window.confirmDeleteTemplate = confirmDeleteTemplate;
window.useTemplate = useTemplate;

// ==================== Context Menu ====================

function showEntryContextMenu(e, entryId) {
  // Remove any existing context menu
  const existingMenu = document.querySelector('.context-menu');
  if (existingMenu) existingMenu.remove();

  const entry = state.entries.find(en => en.id === entryId);
  if (!entry) return;

  const menu = document.createElement('div');
  menu.className = 'context-menu fixed bg-gray-800 border border-gray-700 rounded-lg shadow-xl py-1 z-50';
  menu.style.left = `${e.clientX}px`;
  menu.style.top = `${e.clientY}px`;

  menu.innerHTML = `
    <button class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2" data-action="execute">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
      </svg>
      ${t('context_menu.execute')}
    </button>
    <button class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2" data-action="edit">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
      </svg>
      ${t('context_menu.edit')}
    </button>
    <button class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2" data-action="copy">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"/>
      </svg>
      ${t('context_menu.copy_path')}
    </button>
    <div class="border-t border-gray-700 my-1"></div>
    <button class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2 text-red-400" data-action="delete">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
      </svg>
      ${t('context_menu.delete')}
    </button>
  `;

  document.body.appendChild(menu);

  // Handle menu actions
  menu.addEventListener('click', async (e) => {
    const action = e.target.closest('button')?.dataset.action;
    if (!action) return;

    menu.remove();

    switch (action) {
      case 'execute':
        handleEntryExecution(entryId);
        break;
      case 'edit':
        editEntry(entryId);
        break;
      case 'copy':
        navigator.clipboard.writeText(entry.target);
        showToast(t('toasts.copy_path'), 'success');
        break;
      case 'delete':
        confirmDeleteEntry(entryId);
        break;
    }
  });

  // Close menu on click outside
  const closeMenu = (e) => {
    if (!menu.contains(e.target)) {
      menu.remove();
      document.removeEventListener('click', closeMenu);
    }
  };
  setTimeout(() => document.addEventListener('click', closeMenu), 0);
}

// ==================== Hotkey Recording ====================

function setupHotkeyRecording(inputId, onUpdate) {
  const input = document.getElementById(inputId);
  if (!input) return;

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Tab') {
      return;
    }

    e.stopPropagation();

    if (e.key === 'Backspace' || e.key === 'Delete' || e.key === 'Escape') {
      e.preventDefault();
      input.value = '';
      if (onUpdate) onUpdate();
      return;
    }

    e.preventDefault();

    const parts = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');

    // Get key name
    let key = e.key;
    if (key === ' ') key = 'Space';
    else if (key.length === 1) key = key.toUpperCase();
    else if (key === 'Control' || key === 'Alt' || key === 'Shift' || key === 'Meta') return;

    parts.push(key);
    input.value = parts.join('+');
    if (onUpdate) onUpdate();
  });
}

async function registerGlobalHotkeyListener() {
  try {
    await listen('hotkey-triggered', async (event) => {
      const entryId = event?.payload?.entry_id;
      if (!entryId) return;
      await handleEntryExecution(entryId);
    });
  } catch (error) {
    console.error('Failed to register global hotkey listener:', error);
  }
}

// ==================== Settings Sync ====================

async function handleSettingsChange() {
  const previousLanguage = getCurrentLanguage();
  const settings = {
    icon_size: parseInt(document.getElementById('setting-icon-size').value),
    sort_strategy: document.getElementById('setting-sort-strategy').value,
    max_results: parseInt(document.getElementById('setting-max-results').value),
    show_path: document.getElementById('setting-show-path').checked,
    show_type_label: document.getElementById('setting-show-type').checked,
    show_description: document.getElementById('setting-show-desc').checked,
    confirm_dangerous_commands: document.getElementById('setting-confirm-dangerous').checked,
    auto_launch: document.getElementById('setting-auto-launch').checked,
    app_hotkey: document.getElementById('setting-app-hotkey').value,
    theme: state.settings?.theme || 'system',
    language: document.getElementById('setting-language').value || DEFAULT_LANGUAGE,
    search_debounce_ms: state.settings?.search_debounce_ms || 150,
  };

  try {
    state.settings = await updateSettings(settings);
    const nextLanguage = getCurrentLanguage();
    if (nextLanguage !== previousLanguage) {
      applyLanguage(nextLanguage);
    } else {
      // Re-render search results with new settings
      renderSearchResults(state.searchResults);
    }
  } catch (error) {
    // Error already shown via toast
  }
}

// ==================== Window Spy ====================

async function handleOpenWindowSpy() {
  try {
    await invoke('open_window_spy');
  } catch (error) {
    const message = error?.message || String(error);
    console.error('Failed to open WindowSpy:', error);
    showToast(t('toasts.window_spy_failed', { error: message }), 'error');
  }
}

// ==================== Browse Dialogs ====================

async function handleBrowseTarget() {
  const type = document.getElementById('entry-type').value;
  let path;

  if (type === 'dir') {
    path = await invoke('open_directory_dialog');
  } else {
    path = await invoke('open_file_dialog');
  }

  if (path) {
    document.getElementById('entry-target').value = path;
  }
}

async function handleBrowseWorkdir() {
  const path = await invoke('open_directory_dialog');
  if (path) {
    document.getElementById('entry-workdir').value = path;
  }
}

async function handleBrowseIcon() {
  const path = await invoke('open_file_dialog');
  if (path) {
    document.getElementById('entry-icon').value = path;
  }
}

// ==================== Data Loading ====================

async function loadSettings() {
  state.settings = await getSettings();
  if (state.settings) {
    updateSettingsUI(state.settings);
  }
}

async function loadEntries() {
  state.entries = await getAllEntries();
}

async function loadHotkeys() {
  state.hotkeys = await getAllHotkeys();
}

async function loadTemplates() {
  state.templates = await getAllTemplates();
}

async function loadAllData() {
  await Promise.all([
    loadSettings(),
    loadEntries(),
    loadHotkeys(),
    loadTemplates(),
  ]);
}

// ==================== Initialization ====================

async function init() {
  applyTranslations(DEFAULT_LANGUAGE);

  // Load all data
  await loadAllData();

  applyLanguage(state.settings?.language || DEFAULT_LANGUAGE);
  await registerGlobalHotkeyListener();

  // Initial search (show all entries)
  handleSearch('');

  // Setup search input
  const searchInput = document.getElementById('search-input');
  const debouncedSearch = debounce(handleSearch, state.settings?.search_debounce_ms || 150);

  searchInput.addEventListener('input', (e) => {
    debouncedSearch(e.target.value);
  });

  // Focus search input by default
  requestAnimationFrame(() => {
    focusSearchInput();
  });

  window.addEventListener('focus', () => {
    focusSearchInput();
  });

  document.addEventListener('mousedown', (e) => {
    if (shouldFocusSearchOnClick(e.target)) {
      focusSearchInput();
    }
  });

  // Setup keyboard navigation
  document.addEventListener('keydown', handleKeyboardNavigation);

  // Window controls
  document.getElementById('btn-minimize').addEventListener('click', () => {
    invoke('minimize_window');
  });

  document.getElementById('btn-maximize').addEventListener('click', () => {
    invoke('toggle_maximize_window');
  });

  document.getElementById('btn-close').addEventListener('click', () => {
    invoke('close_window');
  });

  document.getElementById('btn-github').addEventListener('click', () => {
    openExternalLink(GITHUB_REPO_URL);
  });

  document.getElementById('btn-about').addEventListener('click', showAboutModal);
  document.getElementById('btn-close-about').addEventListener('click', closeAboutModal);
  document.getElementById('btn-help').addEventListener('click', showHelpModal);
  document.getElementById('btn-close-help').addEventListener('click', closeHelpModal);

  // Settings modal
  document.getElementById('btn-settings').addEventListener('click', showSettingsModal);
  document.getElementById('btn-close-settings').addEventListener('click', closeSettingsModal);

  // Settings tabs
  document.querySelectorAll('.sidebar-item').forEach(item => {
    item.addEventListener('click', () => {
      switchSettingsTab(item.dataset.tab);
    });
  });

  const entriesSearchInput = document.getElementById('entries-search-input');
  if (entriesSearchInput) {
    entriesSearchInput.value = state.entriesSearchQuery;
    entriesSearchInput.addEventListener('input', handleEntriesSearchInput);
  }

  const entriesSelectAll = document.getElementById('entries-select-all');
  if (entriesSelectAll) {
    entriesSelectAll.addEventListener('change', (e) => {
      handleSelectAllEntries(e.target.checked);
    });
  }

  const deleteSelectedEntriesBtn = document.getElementById('btn-delete-selected-entries');
  if (deleteSelectedEntriesBtn) {
    deleteSelectedEntriesBtn.addEventListener('click', handleDeleteSelectedEntries);
  }

  // Entry modal
  document.getElementById('btn-add-entry').addEventListener('click', showAddEntryModal);
  document.getElementById('btn-close-entry-modal').addEventListener('click', closeEntryModal);
  document.getElementById('btn-cancel-entry').addEventListener('click', closeEntryModal);
  document.getElementById('btn-test-entry').addEventListener('click', handleEntryFormTest);
  document.getElementById('entry-form').addEventListener('submit', handleEntryFormSubmit);
  document.getElementById('entry-type').addEventListener('change', updateEntryFormFields);
  document.getElementById('entry-target').addEventListener('input', syncEntryFormRequirements);
  document.getElementById('entry-script-content').addEventListener('input', syncEntryFormRequirements);
  document.getElementById('entry-hotkey-position').addEventListener('change', (e) => {
    e.target.dataset.touched = 'true';
  });
  document.getElementById('entry-hotkey-detect-hidden').addEventListener('change', (e) => {
    e.target.dataset.touched = 'true';
  });
  document.getElementById('btn-window-spy').addEventListener('click', handleOpenWindowSpy);
  document.getElementById('btn-browse-target').addEventListener('click', handleBrowseTarget);
  document.getElementById('btn-browse-workdir').addEventListener('click', handleBrowseWorkdir);
  document.getElementById('btn-browse-icon').addEventListener('click', handleBrowseIcon);
  document.getElementById('btn-clear-entry-hotkey').addEventListener('click', () => {
    document.getElementById('entry-hotkey').value = '';
  });
  setupHotkeyRecording('entry-hotkey');
  setupHotkeyRecording('setting-app-hotkey', handleSettingsChange);
  document.getElementById('btn-clear-app-hotkey').addEventListener('click', () => {
    document.getElementById('setting-app-hotkey').value = '';
    handleSettingsChange();
  });

  // Template modal
  const addTemplateButton = document.getElementById('btn-add-template');
  if (addTemplateButton) {
    addTemplateButton.addEventListener('click', showAddTemplateModal);
  }
  const closeTemplateButton = document.getElementById('btn-close-template-modal');
  if (closeTemplateButton) {
    closeTemplateButton.addEventListener('click', closeTemplateModal);
  }
  const cancelTemplateButton = document.getElementById('btn-cancel-template');
  if (cancelTemplateButton) {
    cancelTemplateButton.addEventListener('click', closeTemplateModal);
  }
  const templateForm = document.getElementById('template-form');
  if (templateForm) {
    templateForm.addEventListener('submit', handleTemplateFormSubmit);
  }
  const addVariableButton = document.getElementById('btn-add-variable');
  if (addVariableButton) {
    addVariableButton.addEventListener('click', addTemplateVariable);
  }
  setupTemplateVariableActions();

  // Use template modal
  document.getElementById('btn-close-use-template').addEventListener('click', closeUseTemplateModal);
  document.getElementById('btn-cancel-use-template').addEventListener('click', closeUseTemplateModal);
  document.getElementById('use-template-form').addEventListener('submit', handleUseTemplateFormSubmit);

  // Confirm modal
  document.getElementById('btn-confirm-cancel').addEventListener('click', closeConfirmModal);

  // Import/Export
  document.getElementById('btn-export').addEventListener('click', exportData);
  document.getElementById('btn-import').addEventListener('click', () => {
    const strategy = document.getElementById('import-strategy').value;
    importData(strategy);
  });

  // Settings changes
  document.getElementById('setting-icon-size').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-sort-strategy').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-max-results').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-show-path').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-show-type').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-show-desc').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-confirm-dangerous').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-auto-launch').addEventListener('change', handleSettingsChange);
  document.getElementById('setting-language').addEventListener('change', handleSettingsChange);

  // Block Alt+Space from opening the system menu inside the app
  document.addEventListener('keydown', (e) => {
    if (e.altKey && (e.code === 'Space' || e.key === ' ' || e.key === 'Spacebar')) {
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    if (e.key === 'Escape') {
      closeAllModals();
    }
  }, true);

  // Close modals on overlay click
  document.querySelectorAll('.modal-overlay').forEach(overlay => {
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) {
        closeAllModals();
      }
    });
  });

  console.log('Opener initialized');
}

// Start the application
document.addEventListener('DOMContentLoaded', init);
