//! Demo binary showcasing Stage 7: Multi-Monitor Topology, DPI Adaptation, Resource Saver, and Automated Release Checksums.

use rawaq_lib::display::{DisplayInfo, DisplayRect, DisplayTopologyManager};
use rawaq_lib::release::ReleaseManager;
use std::fs;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة تعدد الشاشات وتوفير الموارد وبصمات الإصدار");
    println!("═══════════════════════════════════════════════════════════════\n");

    // 1. Multi-Monitor Setup
    println!("1. محاكاة طوبولوجيا الشاشات المتعددة (Multi-Monitor Topology):");
    let mut manager = DisplayTopologyManager::new();

    let d1 = DisplayInfo {
        id: "disp_primary".to_string(),
        name: "شاشة الحاسوب المدمجة (Primary 1080p)".to_string(),
        is_primary: true,
        bounds: DisplayRect::new(0, 0, 1920, 1080),
        scale_factor: 1.0,
    };
    let d2 = DisplayInfo {
        id: "disp_external".to_string(),
        name: "شاشة خارجية فائقة الدقة (External 4K)".to_string(),
        is_primary: false,
        bounds: DisplayRect::new(1920, 0, 3840, 2160),
        scale_factor: 1.5,
    };

    manager.register_display(d1.clone());
    manager.register_display(d2.clone());

    println!("   ✓ تم تسجيل شاشتين بنجاح:");
    println!("     - [D1] {} (1920x1080 @ 0,0)", d1.name);
    println!("     - [D2] {} (3840x2160 @ 1920,0)", d2.name);

    // Place Box 101 on D2
    let initial_rect = manager
        .place_box(101, "disp_external", 2400, 350, 600, 400)
        .expect("Place box on D2");
    println!("   ✓ تم تعيين الصندوق #101 على الشاشة الخارجية عند الإحداثيات: ({}, {}) بحجم {}x{}",
        initial_rect.x, initial_rect.y, initial_rect.width, initial_rect.height);

    // 2. Unplug External Monitor
    println!("\n2. تجربة فصل الشاشة الخارجية (Unplug Monitor):");
    let unplug_report = manager
        .unregister_display("disp_external")
        .expect("Unregister external display");
    println!("   ✓ تم فصل الشاشة الخارجية بنجاح.");
    println!("   ✓ تقرير التكيّف التلقائي: تم نقل {} صندوق إلى الشاشة الأساسية المتبقية.",
        unplug_report.relocated_boxes_count);

    let relocated = manager.get_placement(101).expect("Placement exists");
    println!("   ✓ الموقع المؤقت للصندوق على الشاشة الأساسية: ({}, {})", relocated.rect.x, relocated.rect.y);
    assert!(d1.bounds.contains_rect(&relocated.rect));
    println!("   ✓ الصندوق محمي بالكامل داخل حدود الشاشة الأساسية ولم يضع في الفراغ!");

    // 3. Reconnect External Monitor
    println!("\n3. تجربة إعادة توصيل الشاشة الخارجية (Reconnect Monitor):");
    let reconnect_report = manager.register_display(d2.clone());
    println!("   ✓ تمت إعادة توصيل الشاشة الخارجية بنجاح.");
    println!("   ✓ تقرير الاستعادة: تمت استعادة {} صندوق إلى شاشته المفضلة.",
        reconnect_report.restored_boxes_count);

    let restored = manager.get_placement(101).expect("Placement exists");
    println!("   ✓ الإحداثيات المستعادة بدقة: ({}, {}) بحجم {}x{}",
        restored.rect.x, restored.rect.y, restored.rect.width, restored.rect.height);
    assert_eq!(restored.rect.x, 2400);
    assert_eq!(restored.rect.y, 350);
    println!("   ✓ عادت الصناديق لمواقعها الأصلية بنسبة 100%!");

    // 4. Resolution & DPI Adaptation
    println!("\n4. تجربة تكيّف تغيير الدقة ومقياس العرض (Resolution & Scale Change):");
    let hd_bounds = DisplayRect::new(0, 0, 1366, 768);
    manager
        .set_display_resolution("disp_primary", hd_bounds, 1.0)
        .expect("Update resolution to 1366x768");
    println!("   ✓ تم تقليص دقة الشاشة الأساسية من 1080p إلى 1366x768.");

    manager
        .place_box(202, "disp_primary", 1300, 700, 300, 200)
        .expect("Place box near bottom-right");
    let box202 = manager.get_placement(202).expect("Placement exists");
    assert!(hd_bounds.contains_rect(&box202.rect));
    println!("   ✓ تم التحقق: الصندوق مقيّد بأمان داخل الحدود ({}, {}) بحجم {}x{}",
        box202.rect.x, box202.rect.y, box202.rect.width, box202.rect.height);

    // 5. Resource Saver Mode
    println!("\n5. تجربة وضع توفير الموارد (Resource Saver Mode):");
    manager.set_box_collapsed(101, true).unwrap();
    manager.set_box_collapsed(202, true).unwrap();
    let saver_report = manager.toggle_resource_saver(true);
    println!("   ✓ تم تفعيل وضع توفير الموارد بنجاح:");
    println!("     - عدد الصناديق المعلقة بصريًا: {}", saver_report.suspended_boxes_count);
    println!("     - الذاكرة البصرية التقديرية الموفرة: {} كيلوبايت (~{} ميجابايت)",
        saver_report.estimated_memory_saved_kb, saver_report.estimated_memory_saved_kb / 1024);

    // 6. Automated GitHub Release & SHA-256 Checksums
    println!("\n6. تجربة توليد والتحقق من بصمات الإصدار (Automated Release SHA-256):");
    let temp_dir = tempfile::tempdir().expect("Create release directory");
    let release_path = temp_dir.path();

    // Create simulated release artifacts
    let windows_msi = release_path.join("rawaq_0.1.0_x64_en-US.msi");
    let windows_exe = release_path.join("rawaq_0.1.0_x64-setup.exe");
    let macos_dmg = release_path.join("Rawaq_0.1.0_universal.dmg");
    let linux_appimage = release_path.join("rawaq_0.1.0_amd64.AppImage");
    let linux_deb = release_path.join("rawaq_0.1.0_amd64.deb");

    fs::write(&windows_msi, b"RAWAG-V0.1.0-WINDOWS-MSI-PACKAGE-MOCK").unwrap();
    fs::write(&windows_exe, b"RAWAG-V0.1.0-WINDOWS-EXE-PACKAGE-MOCK").unwrap();
    fs::write(&macos_dmg, b"RAWAG-V0.1.0-MACOS-DMG-PACKAGE-MOCK").unwrap();
    fs::write(&linux_appimage, b"RAWAG-V0.1.0-LINUX-APPIMAGE-PACKAGE-MOCK").unwrap();
    fs::write(&linux_deb, b"RAWAG-V0.1.0-LINUX-DEBIAN-PACKAGE-MOCK").unwrap();

    let (manifest, artifacts) = ReleaseManager::generate_sha256sums_from_dir(release_path)
        .expect("Generate release checksums");
    println!("   ✓ تم فحص وتوليد البصمات لـ {} حزم تنفيذية لجميع الأنظمة الثلاثة:", artifacts.len());
    for a in &artifacts {
        println!("     - [{}] {} (SHA-256: {}...)", a.platform, a.filename, &a.sha256[0..16]);
    }

    let report = ReleaseManager::verify_sha256sums(&manifest, release_path)
        .expect("Verify checksums against files");
    assert!(report.is_valid);
    println!("   ✓ نتيجة التحقق الصارم للبصمات: متطابقة 100% (Verified: {} files)", report.verified_files.len());

    // Fault injection check
    fs::write(&windows_msi, b"TAMPERED-BYTE-MODIFIED").unwrap();
    let tampered_result = ReleaseManager::verify_sha256sums(&manifest, release_path);
    assert!(tampered_result.is_err());
    println!("   ✓ كشف فوري للعبث أو تلف الحزمة: تم رصد عدم تطابق البصمة ورفض الملف تلقائيًا.");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع تجارب تعدد الشاشات وتوفير الموارد وبصمات الإصدار 100%");
    println!("═══════════════════════════════════════════════════════════════");
}
