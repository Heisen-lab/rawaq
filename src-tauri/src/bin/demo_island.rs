use rawaq_lib::island::{
    CapsuleVisibility, ClipboardContentKind, FullscreenDetector, QuickShelfManager,
    SmartClipboardService,
};
use rawaq_lib::organizer::compute_file_sha256;
use rawaq_lib::security::CommandGate;
use rawaq_lib::storage::{CaptureNote, Storage};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة كبسولة الشاشة والدرج المؤقت والحافظة ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let temp_dir = tempdir().expect("فشل في إنشاء المجلد التجريبي المؤقت");
    let base_path = temp_dir.path();

    // 1. إنشاء ملفات واقعية على القرص
    let f1 = base_path.join("مخطط_المشروع_2026.docx");
    let f2 = base_path.join("تصميم_الواجهة_v2.png");
    let f3 = base_path.join("تقرير_الأمان_المعتمد.pdf");

    {
        let mut file = File::create(&f1).unwrap();
        file.write_all(b"Document Content for Project Schema 2026")
            .unwrap();
    }
    {
        let mut file = File::create(&f2).unwrap();
        file.write_all(b"Design UI Mockup PNG Bytes 123456789")
            .unwrap();
    }
    {
        let mut file = File::create(&f3).unwrap();
        file.write_all(b"Verified Security Audit Report PDF Content")
            .unwrap();
    }

    let f1_hash = compute_file_sha256(&f1).unwrap();
    let f2_hash = compute_file_sha256(&f2).unwrap();
    let f3_hash = compute_file_sha256(&f3).unwrap();

    println!("1. تم إنشاء 3 ملفات واقعية على سطح المكتب:");
    println!(
        "   - [DOCX] مخطط_المشروع_2026.docx (بصمة: {}...)",
        &f1_hash[..16]
    );
    println!(
        "   - [PNG]  تصميم_الواجهة_v2.png (بصمة: {}...)",
        &f2_hash[..16]
    );
    println!(
        "   - [PDF]  تقرير_الأمان_المعتمد.pdf (بصمة: {}...)\n",
        &f3_hash[..16]
    );

    // 2. تجربة درج الملفات المؤقت (Quick Shelf) مع إثبات الصفرية المطلقة للمساس بالقرص
    println!("2. تجربة درج الملفات المؤقت (Quick Shelf):");
    let shelf = QuickShelfManager::new();

    let paths = vec![
        f1.to_string_lossy().to_string(),
        f2.to_string_lossy().to_string(),
        f3.to_string_lossy().to_string(),
    ];

    let staged = shelf
        .add_paths(&paths)
        .expect("فشل في إضافة المراجع إلى الدرج");
    println!(
        "   ✓ تم سحب وإفلات {} ملفات في الدرج المؤقت بنجاح:",
        staged.len()
    );
    for item in &staged {
        println!(
            "     - {} (الحجم: {} بايت، معرّف المرجع: {})",
            item.file_name, item.file_size, item.id
        );
    }

    // التأكد من أن الملفات على القرص لم تُمس
    assert_eq!(compute_file_sha256(&f1).unwrap(), f1_hash);
    assert_eq!(compute_file_sha256(&f2).unwrap(), f2_hash);
    assert_eq!(compute_file_sha256(&f3).unwrap(), f3_hash);
    println!("   ✓ إثبات الأمان: جميع الملفات على القرص بقيت دون أدنى تعديل على بصماتها.");

    // إزالة عنصر واحد من الدرج المؤقت
    let removed_id = &staged[0].id;
    let did_remove = shelf.remove_item(removed_id);
    assert!(did_remove);
    assert_eq!(shelf.count(), 2);
    assert!(
        f1.exists(),
        "حذف المرجع من الدرج المؤقت يجب ألا يمس الملف الأصلي على القرص!"
    );
    assert_eq!(compute_file_sha256(&f1).unwrap(), f1_hash);
    println!("   ✓ إزالة عنصر من الدرج: تم حذف المرجع وبقي الملف الأصلي على القرص 100% دون مساس.");

    // إفراغ الدرج المؤقت بالكامل
    let cleared = shelf.clear();
    assert_eq!(cleared, 2);
    assert_eq!(shelf.count(), 0);
    assert!(f1.exists());
    assert!(f2.exists());
    assert!(f3.exists());
    println!(
        "   ✓ إفراغ الدرج بالكامل: جميع الملفات الأصلية ما زالت موجودة وسليمة على القرص تمامًا.\n"
    );

    // 3. تجربة كشف ملء الشاشة والإخفاء التلقائي (Fullscreen Auto-Hide)
    println!("3. تجربة كشف ملء الشاشة (Fullscreen Auto-Hide):");
    let detector = FullscreenDetector::new();
    println!(
        "   - الحالة المبدئية للكبسولة: {:?}",
        detector.get_state().visibility
    );

    // طي الكبسولة إلى شريط كبسولي مصغر
    detector.toggle_collapse();
    println!(
        "   - بعد الطي إلى كبسولة مصغرة: {:?}",
        detector.get_state().visibility
    );

    // محاكاة فتح لعبة أو مشغل فيديو بملء الشاشة
    let fs_enter = detector.set_fullscreen_active(true);
    println!("   ✓ تم كشف ملء الشاشة (Fullscreen Enter):");
    println!(
        "     الحالة الحالية: {:?} | الإخفاء التلقائي مفعّل: {}",
        fs_enter.visibility, fs_enter.is_fullscreen_active
    );
    assert_eq!(fs_enter.visibility, CapsuleVisibility::HiddenFullscreen);

    // محاكاة الخروج من ملء الشاشة
    let fs_exit = detector.set_fullscreen_active(false);
    println!("   ✓ تم الخروج من ملء الشاشة (Fullscreen Exit):");
    println!(
        "     استعادة الحالة السابقة تلقائيًا: {:?} بنجاح!\n",
        fs_exit.visibility
    );
    assert_eq!(fs_exit.visibility, CapsuleVisibility::Collapsed);

    // 4. تجربة إجراءات الحافظة الذكية مع التنظيف الأمني الصارم
    println!("4. تجربة إجراءات الحافظة الذكية (Smart Clipboard Opt-in):");

    // أ) التحقق من احترام الخصوصية عند عدم التفعيل
    let privacy_check = SmartClipboardService::process_text("https://secret.com", false).unwrap();
    assert!(privacy_check.is_none());
    println!("   ✓ احترام الخصوصية: عند عدم تفعيل الميزة لا تتم معالجة الحافظة إطلاقًا.");

    // ب) فحص وحظر رابط خبيث (javascript: injection)
    let bad_link = "javascript:alert('Stolen Token')";
    let bad_res = SmartClipboardService::process_text(bad_link, true)
        .unwrap()
        .unwrap();
    println!("   ✓ فحص محتوى خطير ({}):", bad_link);
    println!(
        "     النتيجة: {:?} (تم الرفض بنجاح: {})",
        bad_res.kind,
        bad_res.rejection_reason.unwrap_or_default()
    );
    assert_eq!(bad_res.kind, ClipboardContentKind::RejectedDangerous);

    // ج) فحص رابط ويب آمن وتنسيقه
    let safe_url = "https://rawaq.app/releases/v0.1.0";
    let safe_res = SmartClipboardService::process_text(safe_url, true)
        .unwrap()
        .unwrap();
    println!("   ✓ فحص رابط آمن ({}):", safe_url);
    println!(
        "     النوع: {:?} | العنوان المقترح: '{}'",
        safe_res.kind, safe_res.title
    );
    assert_eq!(safe_res.kind, ClipboardContentKind::SafeUrl);

    // د) فحص نص عربي مختلط مع وسوم HTML وحفظه في SQLite
    let mixed_note = "<b>اجتماع تخطيط المرحلة 5</b> <span style='color:red'>مع فريق التطوير</span> في تمام الساعة 2 ظهرًا";
    let note_res = SmartClipboardService::process_text(mixed_note, true)
        .unwrap()
        .unwrap();
    println!("   ✓ فحص نص مع وسوم HTML:");
    println!("     النص بعد التطهير: '{}'", note_res.sanitized_content);
    assert_eq!(note_res.kind, ClipboardContentKind::PlainText);

    // حفظ الملاحظة الملتقطة في قاعدة بيانات رَوَاق المحلية
    let db_path = base_path.join("island_demo.db");
    let storage = Storage::open(&db_path).unwrap();
    let note_entry = CaptureNote {
        id: None,
        content: CommandGate::sanitize_text_payload(&note_res.sanitized_content),
        note_type: "clipboard".to_string(),
        tags: "[\"island\", \"demo\"]".to_string(),
        source_url: None,
        created_at: None,
    };
    let note_id = storage.insert_capture_note(&note_entry).unwrap();
    let retrieved = storage.get_capture_note(note_id).unwrap();
    assert_eq!(retrieved.content, note_res.sanitized_content);
    println!(
        "   ✓ تم حفظ الملاحظة المطهرة في قاعدة بيانات SQLite المحلية بنجاح [ID: {}].",
        note_id
    );

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع تجارب كبسولة الشاشة والدرج المؤقت والحافظة 100%");
    println!("═══════════════════════════════════════════════════════════════");
}
