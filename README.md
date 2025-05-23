# 🧼 CleanCraft

[🇬🇧 English](#🇬🇧-english-version) | [🇦🇿 Azərbaycan](#🇦🇿-azərbaycan-versiyası)

---

## 🇦🇿 Azərbaycan Versiyası

**CleanCraft** — Tauri + React texnologiyaları əsasında hazırlanmış yüngül və effektiv **cross-platform sistem təmizləyici** tətbiqdir. Bu tətbiq istifadəçilərə sistem cache fayllarını və müvəqqəti sənədləri silməklə daha sürətli və təmiz bir kompüter təcrübəsi təqdim edir.

### 🚀 Xüsusiyyətlər

- ⚡ **Native sürət və performans** - Rust backend sayəsində yüksək performans
- 🧼 **Hərtərəfli təmizləmə** - Cache, müvəqqəti fayllar və sistem zibilləri
- 🔒 **Təhlükəsiz əməliyyatlar** - Məhdudlaşdırılmış və təhlükəsiz sistem girişi
- 💻 **Cross-platform dəstək** - macOS, Windows və Linux
- 🧑‍💻 **Açıq mənbə** - İstədiyin kimi dəyişdir və genişləndir
- 🎨 **Modern UI** - React əsaslı intuitiv interfeys

### 📁 Təmizləmə Sahələri

- **macOS:**
  - `~/Library/Caches/` - Tətbiq cache faylları
  - `/tmp/` - Müvəqqəti sistem faylları
  - `~/Downloads/` - Köhnə yüklənmələr (istəyə görə)
  
- **Windows:**
  - `%TEMP%` - Müvəqqəti fayllar
  - `%APPDATA%/Local/Temp` - İstifadəçi müvəqqəti faylları
  - Recycle Bin təmizləməsi

- **Linux:**
  - `/tmp/` - Sistem müvəqqəti faylları
  - `~/.cache/` - İstifadəçi cache faylları

### 📦 Texnologiyalar

- **Tauri** - Native cross-platform tətbiq framework
- **Rust** - Performanslı və təhlükəsiz backend
- **React + Vite** - Interaktiv və sürətli frontend
- **TypeScript** - Tip təhlükəsizliyi

### 🔧 Quraşdırma

#### 1. Əvvəlcədən lazım olanlar:

- **Əməliyyat sistemi:** macOS, Windows, və ya Linux
- **[Rust](https://rustup.rs)** - Backend üçün
- **[Node.js](https://nodejs.org/)** (v16+) - Frontend üçün
- **Tauri CLI** (qlobal):
  ```bash
  npm install -g @tauri-apps/cli
  ```

#### 2. Layihəni klonlayın:

```bash
git clone https://github.com/yourusername/cleancraft.git
cd cleancraft
```

#### 3. Dependencies quraşdırın:

```bash
# Frontend dependencies
npm install

# Rust dependencies (avtomatik yüklənəcək)
cargo build
```

#### 4. Development rejimində işə salın:

```bash
npm run tauri dev
```

#### 5. Production build:

```bash
npm run tauri build
```

### 🎯 İstifadə

1. **Tətbiqi açın** - CleanCraft.app (macOS) və ya CleanCraft.exe (Windows)
2. **Scan düyməsini basın** - Sistem təhlil ediləcək
3. **Təmizləmə sahələrini seçin** - Nəyi silmək istədiyinizi müəyyən edin
4. **Clean düyməsini basın** - Seçilmiş fayllar silinəcək
5. **Nəticəni yoxlayın** - Azad edilmiş yaddaş həcmini görün

### 🤝 Töhfə vermək

Töhfələr xoş qarşılanır! Zəhmət olmasa:

1. Bu repo-nu fork edin
2. Feature branch yaradın (`git checkout -b feature/AmazingFeature`)
3. Dəyişikliklərinizi commit edin (`git commit -m 'Add some AmazingFeature'`)
4. Branch-ı push edin (`git push origin feature/AmazingFeature`)
5. Pull Request açın

### 📄 Lisenziya

Bu layihə MIT lisenziyası altında paylaşılır. Ətraflı məlumat üçün [LICENSE](LICENSE) faylına baxın.

---

## 🇬🇧 English Version

**CleanCraft** is a lightweight and efficient **cross-platform system cleaner** built with Tauri + React technologies. This application provides users with a faster and cleaner computer experience by removing system cache files and temporary documents.

### 🚀 Features

- ⚡ **Native speed and performance** - High performance through Rust backend
- 🧼 **Comprehensive cleaning** - Cache, temporary files, and system junk
- 🔒 **Safe and secure operations** - Restricted and secure system access
- 💻 **Cross-platform support** - macOS, Windows, and Linux
- 🧑‍💻 **Open source** - Modify and extend as you wish
- 🎨 **Modern UI** - React-based intuitive interface

### 📁 Cleaning Areas

- **macOS:**
  - `~/Library/Caches/` - Application cache files
  - `/tmp/` - Temporary system files
  - `~/Downloads/` - Old downloads (optional)
  
- **Windows:**
  - `%TEMP%` - Temporary files
  - `%APPDATA%/Local/Temp` - User temporary files
  - Recycle Bin cleaning

- **Linux:**
  - `/tmp/` - System temporary files
  - `~/.cache/` - User cache files

### 📦 Technologies

- **Tauri** - Native cross-platform app framework
- **Rust** - High-performance and secure backend
- **React + Vite** - Interactive and fast frontend
- **TypeScript** - Type safety

### 🔧 Installation

#### 1. Prerequisites:

- **Operating System:** macOS, Windows, or Linux
- **[Rust](https://rustup.rs)** - For backend
- **[Node.js](https://nodejs.org/)** (v16+) - For frontend
- **Tauri CLI** (global):
  ```bash
  npm install -g @tauri-apps/cli
  ```

#### 2. Clone the repository:

```bash
git clone https://github.com/yourusername/cleancraft.git
cd cleancraft
```

#### 3. Install dependencies:

```bash
# Frontend dependencies
npm install

# Rust dependencies (will be installed automatically)
cargo build
```

#### 4. Run in development mode:

```bash
npm run tauri dev
```

#### 5. Build for production:

```bash
npm run tauri build
```

### 🎯 Usage

1. **Open the application** - CleanCraft.app (macOS) or CleanCraft.exe (Windows)
2. **Click Scan button** - System will be analyzed
3. **Select cleaning areas** - Choose what you want to clean
4. **Click Clean button** - Selected files will be removed
5. **Check results** - See the amount of freed storage space

### 🛡️ Safety Features

- **Whitelist approach** - Only predefined safe directories are cleaned
- **Confirmation dialogs** - User confirmation before deletion
- **Detailed logs** - Track what files are being removed
- **Backup option** - Option to backup files before deletion (coming soon)

### 📊 System Requirements

| Platform | Minimum Requirements |
|----------|---------------------|
| **macOS** | macOS 10.15+ |
| **Windows** | Windows 10+ |
| **Linux** | Ubuntu 18.04+ / equivalent |
| **RAM** | 4GB recommended |
| **Storage** | 50MB free space |

### 🐛 Troubleshooting

**Common Issues:**

- **Permission denied**: Run the app with administrator privileges
- **Build fails**: Ensure Rust and Node.js are properly installed
- **App won't start**: Check system compatibility

**Getting Help:**
- Check [Issues](https://github.com/yourusername/cleancraft/issues) for known problems
- Open a new issue with detailed information
- Join our [Discord community](https://discord.gg/cleancraft) (if available)

### 🤝 Contributing

Contributions are welcome! Please:

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### 📝 Development Setup

For contributors who want to set up the development environment:

```bash
# Clone and navigate
git clone https://github.com/yourusername/cleancraft.git
cd cleancraft

# Install all dependencies
npm install
cargo build

# Start development server
npm run tauri dev

# Run tests
npm test
cargo test

# Format code
npm run format
cargo fmt
```

### 🗺️ Roadmap

- [ ] **v1.1** - Backup functionality before deletion
- [ ] **v1.2** - Scheduled cleaning tasks  
- [ ] **v1.3** - Plugin system for custom cleaners
- [ ] **v2.0** - Cloud storage cleaning integration
- [ ] **v2.1** - AI-powered smart cleaning suggestions

### 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

### 🙏 Acknowledgments

- Tauri team for the amazing framework
- React community for the powerful UI library
- All contributors who help improve CleanCraft

---

**Made with ❤️ by the Eshgin Farzali**

[⬆ Back to top](#🧼-cleancraft)