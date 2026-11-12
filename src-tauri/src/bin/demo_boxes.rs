use rawaq_lib::boxes::{BoxService, FolderState};
use rawaq_lib::storage::Storage;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn hash_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("فشل في قراءة الملف لحساب البصمة");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة صناديق سطح المكتب ووضع الكبسولة      ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let temp_root = tempdir().expect("فشل في إنشاء المجلد التجريبي");
    let db_path = temp_root.path().join("rawaq_boxes.db");
    let storage = Storage::open(&db_path).expect("فشل في فتح قاعدة البيانات");

    // 1. إنشاء مجلد فعلي على القرص بملفات عربية وإنجليزية حقيقية
    let mapped_folder = temp_root.path().join("مشاريع_سطح_المكتب_2026");
    fs::create_dir_all(&mapped_folder).unwrap();

    let file1 = mapped_folder.join("تقرير_الميزانية_السنوي_2026(Final).pdf");
    let file2 = mapped_folder.join("Rawaq_Architecture_Notes.md");
    let file3 = mapped_folder.join("شعار_المشروع_Vector.png");
    let subfolder = mapped_folder.join("ملفات_مؤرشفة");
    fs::create_dir_all(&subfolder).unwrap();

    {
        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(
            b"PDF Content for Rawaq financial report 2026 \xD8\xB9\xD8\xB1\xD8\xA8\xD9\x8A",
        )
        .unwrap();
    }
    {
        let mut f2 = File::create(&file2).unwrap();
        f2.write_all(b"# Architecture Notes: Virtual Mapping by default")
            .unwrap();
    }
    {
        let mut f3 = File::create(&file3).unwrap();
        f3.write_all(b"PNG Header 123456789").unwrap();
    }

    let file1_hash_before = hash_file(&file1);
    let file2_hash_before = hash_file(&file2);

    println!("1. تم إنشاء مجلد المستخدم الفعلي على القرص وفحصه:");
    println!("   المسار: {}", mapped_folder.display());
    println!("   بصمة SHA-256 للملف الأول: {}\n", file1_hash_before);

    // 2. ربط المجلد افتراضيًا بصندوق في رَوَاق دون نقل أي ملف
    println!("2. ربط المجلد افتراضيًا بصندوق في رَوَاق (Virtual Folder Mapping):");
    let box1 = BoxService::create_box(
        &storage,
        "مشاريع سطح المكتب 2026".to_string(),
        mapped_folder.to_string_lossy().to_string(),
        Some("grid".to_string()),
        Some("name".to_string()),
    )
    .expect("فشل في إنشاء الصندوق");

    println!(
        "   ✓ تم إنشاء الصندوق [ID: {}] بنجاح في قاعدة البيانات",
        box1.id.unwrap()
    );
    println!(
        "   ✓ نوع العرض الافتراضي: {} | الترتيب الافتراضي: {}",
        box1.display_mode, box1.sort_order
    );

    // 3. فحص المحتويات وعرض العناصر
    println!("\n3. قراءة محتويات المجلد المربوط بأمان وعرض العناصر:");
    let state = BoxService::scan_folder_contents(&box1.folder_path, &box1.sort_order);
    match state {
        FolderState::Accessible { items } => {
            println!("   ✓ حالة المجلد: متاح وسليم (Accessible)");
            println!("   ✓ عدد العناصر المكتشفة: {}", items.len());
            for (idx, item) in items.iter().enumerate() {
                let kind = if item.is_dir {
                    "📁 مجلد"
                } else {
                    "📄 ملف"
                };
                println!(
                    "     [{}] {} : {} ({} بايت)",
                    idx + 1,
                    kind,
                    item.name,
                    item.size_bytes
                );
            }
        }
        _ => panic!("يجب أن يكون المجلد متاحًا!"),
    }

    // 4. تجربة وضع الكبسولة (Capsule Mode)
    println!("\n4. تجربة وضع الكبسولة (Capsule Mode):");
    let mut box_collapsed = box1.clone();
    box_collapsed.is_capsule_collapsed = true;
    BoxService::update_box(&storage, &box_collapsed).expect("فشل في تعديل الصندوق إلى كبسولة");

    let fetched = storage.get_desktop_box(box1.id.unwrap()).unwrap();
    println!(
        "   ✓ تم طيّ الصندوق إلى كبسولة مضغوطة: is_capsule_collapsed = {}",
        fetched.is_capsule_collapsed
    );

    box_collapsed.is_capsule_collapsed = false;
    BoxService::update_box(&storage, &box_collapsed).expect("فشل في إعادة تمدد الصندوق");
    println!("   ✓ تم استعادة تمدد الصندوق الكامل: is_capsule_collapsed = false");

    // 5. تجربة دمج مجموعات الصناديق (Box Groups)
    println!("\n5. دمج عدة صناديق في مجموعة تبويبات واحدة (Box Group):");
    let box2 = BoxService::create_box(
        &storage,
        "الصور والتصاميم 🎨".to_string(),
        temp_root
            .path()
            .join("pictures")
            .to_string_lossy()
            .to_string(),
        Some("list".to_string()),
        Some("date".to_string()),
    )
    .unwrap();

    let box3 = BoxService::create_box(
        &storage,
        "أكواد ومشاريع Rust 🦀".to_string(),
        temp_root.path().join("code").to_string_lossy().to_string(),
        Some("grid".to_string()),
        Some("size".to_string()),
    )
    .unwrap();

    let group_id = BoxService::merge_boxes_into_group(
        &storage,
        "مجموعة العمل والمطورين",
        &[box1.id.unwrap(), box2.id.unwrap(), box3.id.unwrap()],
    )
    .expect("فشل في دمج المجموعات");

    println!("   ✓ تم إنشاء المجموعة [ID: {}] بنجاح", group_id);
    let members = storage.list_boxes_by_group(group_id).unwrap();
    println!(
        "   ✓ الصناديق المدمجة في التبويب ({} صناديق):",
        members.len()
    );
    for m in &members {
        println!(
            "     - [ID: {}] {} (عرض: {}, ترتيب: {})",
            m.id.unwrap(),
            m.name,
            m.display_mode,
            m.sort_order
        );
    }

    // فك المجموعة
    BoxService::dissolve_group(&storage, group_id).expect("فشل في فك المجموعة");
    println!("   ✓ تم فك المجموعة بنجاح:");
    let b1_after = storage.get_desktop_box(box1.id.unwrap()).unwrap();
    let b2_after = storage.get_desktop_box(box2.id.unwrap()).unwrap();
    println!(
        "     - استعاد الصندوق الأول استقلاليته: group_id = {:?}",
        b1_after.group_id
    );
    println!(
        "     - استعاد الصندوق الثاني استقلاليته: group_id = {:?}",
        b2_after.group_id
    );

    // 6. محاكاة فصل قرص خارجي أو فقدان المجلد (Graceful Disconnection)
    println!("\n6. محاكاة فصل قرص خارجي (Graceful Disconnect State):");
    let disconnected_target = temp_root.path().join("مشاريع_مفصولة_مؤقتًا");
    fs::rename(&mapped_folder, &disconnected_target).unwrap();

    let scan_disconnected = BoxService::scan_folder_contents(&box1.folder_path, &box1.sort_order);
    match scan_disconnected {
        FolderState::MissingOrDisconnected { path, message } => {
            println!("   ✓ تم رصد انقطاع المجلد بأمان دون أي انهيار (Crash):");
            println!("     المسار المفقود: {}", path);
            println!("     رسالة التنبيه الآمن: {}", message);
        }
        _ => panic!("يجب رصد انقطاع المجلد!"),
    }
    // التأكد من بقاء الصندوق في قاعدة البيانات
    assert!(storage.get_desktop_box(box1.id.unwrap()).is_ok());
    println!("   ✓ الصندوق محتفظ به في قاعدة البيانات بكامل إعداداته ولم يُحذف");

    // استعادة المسار
    fs::rename(&disconnected_target, &mapped_folder).unwrap();

    // 7. حذف الصندوق من رَوَاق والتأكد الصارم من عدم مساس الملفات على القرص
    println!("\n7. التأكد الصارم من ميثاق أمان الملفات (Never Modifies Disk Files):");
    BoxService::delete_box(&storage, box1.id.unwrap()).expect("فشل في حذف الصندوق");
    println!("   ✓ تم حذف الصندوق من برنامج رَوَاق");

    assert!(mapped_folder.exists(), "المجلد على القرص يجب ألا يُحذف أبدًا!");
    assert!(file1.exists(), "الملف الأول يجب ألا يُحذف أبدًا!");
    assert!(file2.exists(), "الملف الثاني يجب ألا يُحذف أبدًا!");
    assert_eq!(
        hash_file(&file1),
        file1_hash_before,
        "محتوى الملف وبصمته لم تتغير بنسبة 100%"
    );
    assert_eq!(
        hash_file(&file2),
        file2_hash_before,
        "محتوى الملف وبصمته لم تتغير بنسبة 100%"
    );

    println!("   ✓ المجلد الفعلي وجميع ملفاته سليمة ومطابقة لبصمتها الأصلية 100%");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع تجارب صناديق سطح المكتب والمجموعات بنسبة 100%  ");
    println!("═══════════════════════════════════════════════════════════════");
}
