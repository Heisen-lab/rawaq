use rawaq_lib::organizer::{compute_file_sha256, SafeOrganizerEngine};
use rawaq_lib::storage::{OrganizeRule, Storage};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة محرك الفرز الآمن والتراجع بضغطة زر    ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let temp_root = tempdir().expect("فشل في إنشاء المجلد التجريبي");
    let desktop_dir = temp_root.path().join("سطح_المكتب_Desktop");
    let docs_dir = temp_root.path().join("المستندات_المنظمة");
    let images_dir = temp_root.path().join("الصور_والتصاميم");

    fs::create_dir_all(&desktop_dir).unwrap();

    // 1. إنشاء ملفات واقعية مبعثرة على سطح المكتب
    let f1 = desktop_dir.join("تقرير_الميزانية_السنوي_2026.pdf");
    let f2 = desktop_dir.join("شعار_المشروع_Vector.png");
    let f3 = desktop_dir.join("عقد_العمل_المعتمد(1).pdf");
    let f4 = desktop_dir.join("notes_ملاحظات.txt");

    {
        let mut file = File::create(&f1).unwrap();
        file.write_all(b"PDF Content for Budget Report 2026 \xD8\xB9\xD8\xB1\xD8\xA8\xD9\x8A")
            .unwrap();
    }
    {
        let mut file = File::create(&f2).unwrap();
        file.write_all(b"PNG Image Header Raw Data 987654321")
            .unwrap();
    }
    {
        let mut file = File::create(&f3).unwrap();
        file.write_all(b"Official Contract 2026 Signed and Verified")
            .unwrap();
    }
    {
        let mut file = File::create(&f4).unwrap();
        file.write_all(b"Plain text notes - unorganized").unwrap();
    }

    let f1_hash = compute_file_sha256(&f1).unwrap();
    let f2_hash = compute_file_sha256(&f2).unwrap();
    let f3_hash = compute_file_sha256(&f3).unwrap();

    println!("1. تم إنشاء 4 ملفات واقعية على سطح المكتب:");
    println!(
        "   - [PDF] تقرير_الميزانية_السنوي_2026.pdf (بصمة: {}...)",
        &f1_hash[..16]
    );
    println!(
        "   - [PNG] شعار_المشروع_Vector.png (بصمة: {}...)",
        &f2_hash[..16]
    );
    println!(
        "   - [PDF] عقد_العمل_المعتمد(1).pdf (بصمة: {}...)",
        &f3_hash[..16]
    );
    println!("   - [TXT] notes_ملاحظات.txt\n");

    // قواعد الفرز المقترحة
    let rules = vec![
        OrganizeRule {
            id: Some(1),
            name: "فرز المستندات والتقارير".to_string(),
            extensions: "pdf,docx,xlsx".to_string(),
            target_folder: docs_dir.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        },
        OrganizeRule {
            id: Some(2),
            name: "فرز الصور والتصاميم".to_string(),
            extensions: "png,jpg,svg".to_string(),
            target_folder: images_dir.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        },
    ];

    // 2. إنشاء تعارض متعمد في المجلد الهدف قبل الفرز لاختبار منع الكتابة فوق الملف
    fs::create_dir_all(&docs_dir).unwrap();
    let colliding_file = docs_dir.join("تقرير_الميزانية_السنوي_2026.pdf");
    let original_existing_content = b"ORIGINAL FILE IN TARGET FOLDER - MUST NEVER BE OVERWRITTEN!";
    {
        let mut file = File::create(&colliding_file).unwrap();
        file.write_all(original_existing_content).unwrap();
    }
    let colliding_hash_before = compute_file_sha256(&colliding_file).unwrap();

    println!("2. تشغيل المعاينة الإلزامية قبل النقل (Dry-Run Preview):");
    let plan =
        SafeOrganizerEngine::compute_dry_run(&desktop_dir, &rules).expect("فشل في حساب المعاينة");

    println!("   ✓ تم حساب خطة المعاينة [ID: {}]:", plan.plan_id);
    println!("   ✓ عدد الملفات المخطط نقلها: {}", plan.moves.len());
    println!("   ✓ عدد العناصر المتخطاة: {}", plan.skipped.len());

    for (idx, m) in plan.moves.iter().enumerate() {
        println!("\n   [{}] {}", idx + 1, m.file_name);
        println!("       المصدر: {}", m.source_path);
        println!("       الوجهة: {}", m.resolved_destination_path);
        println!("       القاعدة: {}", m.rule_name);
        if m.collision_detected {
            println!("       ⚠️ تم كشف تعارض وحلّه باسم آمن جديد منعًا لمسح الملف الأصلي!");
        }
    }

    // 3. التحقق من عدم حدوث أي تعديل على القرص أثناء المعاينة
    assert_eq!(
        compute_file_sha256(&colliding_file).unwrap(),
        colliding_hash_before
    );
    assert_eq!(compute_file_sha256(&f1).unwrap(), f1_hash);
    println!("\n   ✓ إثبات الأمان: لم يتغير أي بايت على القرص أثناء المعاينة (Zero Mutations).");

    // 4. تنفيذ الخطة باعتماد ميثاق الأمان الصارم
    println!("\n3. تنفيذ الفرز الذري بقاعدة «النسخ -> التحقق -> التسجيل -> إزالة الأصل»:");
    let db_path = temp_root.path().join("organizer_test.db");
    let storage = Storage::open(&db_path).unwrap();

    let exec_report =
        SafeOrganizerEngine::execute_plan(&storage, &plan, None).expect("فشل في تنفيذ خطة الفرز");

    println!("   ✓ اكتمل التنفيذ بنجاح!");
    println!("   ✓ معرّف الدفعة (Batch ID): {}", exec_report.batch_id);
    println!(
        "   ✓ العمليات الناجحة: {} | العمليات المتعذرة: {}",
        exec_report.successful, exec_report.failed
    );

    // التأكد من أن الملف القديم في الوجهة لم يُمسح
    assert_eq!(
        fs::read(&colliding_file).unwrap(),
        original_existing_content
    );
    println!("   ✓ التأكيد الصارم: الملف السابق في الوجهة لم يُكتب فوقه إطلاقًا!");

    // التأكد من نقل الملف الجديد بالاسم الآمن البديل
    let resolved_moved_file = docs_dir.join("تقرير_الميزانية_السنوي_2026 (1).pdf");
    assert!(resolved_moved_file.exists());
    assert_eq!(compute_file_sha256(&resolved_moved_file).unwrap(), f1_hash);
    println!(
        "   ✓ تم نقل التقرير بنجاح إلى: تقرير_الميزانية_السنوي_2026 (1).pdf مع تطابق البصمة 100%"
    );

    // 5. التراجع الكامل بنقرة واحدة (One-Click Undo)
    println!("\n4. تجربة التراجع الكامل بنقرة زر (One-Click Undo):");
    let undo_report = SafeOrganizerEngine::undo_batch(&storage, &exec_report.batch_id)
        .expect("فشل في التراجع عن الدفعة");

    println!("   ✓ تم تنفيذ التراجع بنجاح!");
    println!(
        "   ✓ عدد الملفات المستعادة إلى سطح المكتب بدقة: {}",
        undo_report.restored
    );

    // التحقق من عودة الملفات إلى سطح المكتب وتطابق بصماتها
    assert!(f1.exists(), "الملف الأول يجب أن يعود لسطح المكتب");
    assert!(f2.exists(), "الملف الثاني يجب أن يعود لسطح المكتب");
    assert!(f3.exists(), "الملف الثالث يجب أن يعود لسطح المكتب");

    assert_eq!(
        compute_file_sha256(&f1).unwrap(),
        f1_hash,
        "تطابق البصمة بعد التراجع 100%"
    );
    assert_eq!(
        compute_file_sha256(&f2).unwrap(),
        f2_hash,
        "تطابق البصمة بعد التراجع 100%"
    );
    assert_eq!(
        compute_file_sha256(&f3).unwrap(),
        f3_hash,
        "تطابق البصمة بعد التراجع 100%"
    );

    // التأكد من حذف النسخ المنقولة من المجلدات المنظمة بعد التراجع
    assert!(!resolved_moved_file.exists());
    assert!(!images_dir.join("شعار_المشروع_Vector.png").exists());

    println!("   ✓ عادت جميع الملفات إلى مساراتها الأصلية على سطح المكتب ببصماتها الدقيقة.");
    println!("   ✓ أُزيلت النسخ المؤقتة من المجلدات الهدف بأمان بعد اكتمال الاستعادة.");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع تجارب الفرز الآمن والتراجع بضغطة زر بنسبة 100%   ");
    println!("═══════════════════════════════════════════════════════════════");
}
