use rawaq_lib::storage::{
    export_backup, import_backup, BoxGroup, CaptureNote, DesktopBox, OrganizeRule, Storage,
    TaskItem, UndoOperation,
};
use std::fs;
use tempfile::tempdir;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة طبقة التخزين المحلي والنسخ الاحتياطي  ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let temp_dir = tempdir().expect("فشل في إنشاء مجلد اختباري");
    let db_path = temp_dir.path().join("rawaq_user_data.db");

    println!("1. فتح قاعدة بيانات SQLite حقيقية على القرص في:");
    println!("   {}\n", db_path.display());

    let storage = Storage::open(&db_path).expect("فشل في فتح قاعدة البيانات وتشغيل الترقيات");

    let wal_mode: String = storage
        .connection()
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("فشل في قراءة journal_mode");
    println!(
        "   ✓ وضع السجل النشط (Journal Mode): {}",
        wal_mode.to_uppercase()
    );

    // 2. إنشاء مجموعة وصناديق سطح المكتب
    println!("\n2. إضافة مجموعة وصناديق سطح مكتب للمستخدم:");
    let group_id = storage
        .insert_box_group(&BoxGroup {
            id: None,
            name: "مشاريع العمل 2026".to_string(),
            created_at: None,
        })
        .expect("فشل في إضافة المجموعة");
    println!(
        "   ✓ تمت إضافة المجموعة بنجاح: [ID: {}] مشاريع العمل 2026",
        group_id
    );

    let box1_id = storage
        .insert_desktop_box(&DesktopBox {
            id: None,
            name: "المستندات والتقارير الرسمية".to_string(),
            folder_path: "C:\\Users\\أحمد\\Desktop\\تقارير_2026".to_string(),
            display_mode: "grid".to_string(),
            is_capsule_collapsed: false,
            sort_order: "date".to_string(),
            position_x: 120.0,
            position_y: 80.0,
            width: 340.0,
            height: 420.0,
            display_id: Some("Monitor-Primary-4K".to_string()),
            group_id: Some(group_id),
            created_at: None,
            updated_at: None,
        })
        .expect("فشل في إضافة الصندوق الأول");

    let box2_id = storage
        .insert_desktop_box(&DesktopBox {
            id: None,
            name: "Research & Code Snippets 🦀".to_string(),
            folder_path: "/home/ahmad/Projects/rawaq-core".to_string(),
            display_mode: "list".to_string(),
            is_capsule_collapsed: true,
            sort_order: "name".to_string(),
            position_x: 480.0,
            position_y: 80.0,
            width: 300.0,
            height: 420.0,
            display_id: Some("Monitor-Secondary".to_string()),
            group_id: Some(group_id),
            created_at: None,
            updated_at: None,
        })
        .expect("فشل في إضافة الصندوق الثاني");

    println!(
        "   ✓ الصندوق 1: [ID: {}] المستندات والتقارير الرسمية (وضع الشبكة - الشاشة الرئيسية)",
        box1_id
    );
    println!(
        "   ✓ الصندوق 2: [ID: {}] Research & Code Snippets 🦀 (وضع الكبسولة المطوية)",
        box2_id
    );

    // 3. إضافة قواعد الفرز والمهام والملاحظات
    println!("\n3. إضافة قواعد الفرز والمهام اليومية والملاحظات الملتقطة:");
    let rule_id = storage
        .insert_organize_rule(&OrganizeRule {
            id: None,
            name: "فرز ملفات PDF تلقائيًا".to_string(),
            extensions: "pdf,djvu".to_string(),
            target_folder: "C:\\Users\\أحمد\\Documents\\كتب_ومراجع".to_string(),
            is_enabled: true,
            created_at: None,
        })
        .expect("فشل في إضافة قاعدة الفرز");
    println!("   ✓ قاعدة الفرز: [ID: {}] فرز ملفات PDF تلقائيًا", rule_id);

    let task_id = storage
        .insert_task(&TaskItem {
            id: None,
            title: "مراجعة تقرير الميزانية النهائي لعام 2026 📊".to_string(),
            description: "التأكد من مطابقة جميع الفواتير والمرفقات".to_string(),
            period: "morning".to_string(),
            is_completed: false,
            tags: "مالية,مهمة_عاجلة,2026".to_string(),
            created_at: None,
            updated_at: None,
        })
        .expect("فشل في إضافة المهمة");
    println!(
        "   ✓ المهمة اليومية: [ID: {}] مراجعة تقرير الميزانية النهائي لعام 2026 📊",
        task_id
    );

    let note_id = storage
        .insert_capture_note(&CaptureNote {
            id: None,
            content: "💡 فكرة: إضافة دعم اختصارات لوحة المفاتيح لاستدعاء الكبسولة العلوية فورًا"
                .to_string(),
            note_type: "text".to_string(),
            tags: "أفكار,UI,UX".to_string(),
            source_url: Some("https://github.com/Heisen-lab/rawaq".to_string()),
            created_at: None,
        })
        .expect("فشل في إضافة الملاحظة");
    println!(
        "   ✓ الملاحظة الملتقطة: [ID: {}] فكرة اختصارات الكبسولة العلوية",
        note_id
    );

    // 4. تسجيل عملية في سجل التراجع
    let undo_id = storage
        .insert_undo_operation(&UndoOperation {
            id: None,
            batch_id: "batch-2026-11-12-001".to_string(),
            source_path: "C:\\Users\\أحمد\\Desktop\\سجل_المبيعات_Q3.xlsx".to_string(),
            destination_path: "C:\\Users\\أحمد\\Documents\\أرشيف\\سجل_المبيعات_Q3.xlsx".to_string(),
            file_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_string(),
            file_size: 2048576,
            status: "COMPLETED".to_string(),
            error_message: None,
            created_at: None,
        })
        .expect("فشل في تسجيل عملية التراجع");
    println!(
        "   ✓ سجل التراجع: [ID: {}] عملية فرز مؤرشفة ببصمة SHA-256",
        undo_id
    );

    // 5. تصدير نسخة احتياطية موثّقة
    println!("\n4. تصدير نسخة احتياطية محلية كاملة موثّقة ببصمة SHA-256:");
    let backup_json = export_backup(&storage).expect("فشل في تصدير النسخة الاحتياطية");
    let backup_file = temp_dir.path().join("rawaq_backup_2026.json");
    fs::write(&backup_file, &backup_json).expect("فشل في حفظ ملف النسخة الاحتياطية");

    let parsed: serde_json::Value = serde_json::from_str(&backup_json).unwrap();
    let checksum = parsed["sha256"].as_str().unwrap_or("unknown");
    let file_size = backup_json.len();
    println!("   ✓ تم إنشاء ملف النسخ الاحتياطي بنجاح: {} بايت", file_size);
    println!("   ✓ بصمة SHA-256 للملف: {}", checksum);

    // 6. استعادة النسخة في قاعدة بيانات جديدة خالية
    println!("\n5. اختبار استيراد النسخة الاحتياطية إلى قاعدة بيانات جديدة خالية:");
    let new_db_path = temp_dir.path().join("rawaq_restored.db");
    let mut fresh_storage = Storage::open(&new_db_path).expect("فشل في فتح قاعدة البيانات الجديدة");
    import_backup(&mut fresh_storage, &backup_json).expect("فشل في استيراد النسخة الاحتياطية");

    let restored_boxes = fresh_storage
        .list_desktop_boxes()
        .expect("فشل في قراءة الصناديق");
    let restored_task = fresh_storage
        .get_task(1)
        .expect("فشل في قراءة المهمة المستعادة");
    println!("   ✓ عدد الصناديق المستعادة بدقة: {}", restored_boxes.len());
    println!("   ✓ المهمة المستعادة: {}", restored_task.title);

    // 7. اختبار رفض حزمة معدّلة أو متلاعب بها
    println!("\n6. اختبار أمان النسخ الاحتياطي: التلاعب المتعمد بمحتوى الملف:");
    let tampered_json = backup_json.replace("المستندات والتقارير الرسمية", "ملف تم التلاعب به");
    let tamper_result = import_backup(&mut fresh_storage, &tampered_json);
    match tamper_result {
        Err(err) => println!("   ✓ تم رصد التلاعب ورفض الاستيراد بأمان: {}", err),
        Ok(_) => panic!("خطأ أمني: تم قبول نسخة متلاعب بها!"),
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع العمليات والتجارب الحية للمرحلة 2 بنسبة 100%   ");
    println!("═══════════════════════════════════════════════════════════════");
}
