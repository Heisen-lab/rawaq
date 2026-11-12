use rawaq_lib::productivity::{CaptureManager, PomodoroState, PomodoroTimer, TaskManager};
use rawaq_lib::search::{SearchableFile, UnifiedSearchEngine};
use rawaq_lib::storage::Storage;
use tempfile::tempdir;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  رَوَاق (Rawaq) — تجربة المهام اليومية، الالتقاط، المؤقت، والبحث ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let temp_dir = tempdir().expect("فشل في إنشاء المجلد التجريبي");
    let db_path = temp_dir.path().join("productivity_demo.db");
    let storage = Storage::open(&db_path).expect("فشل في فتح قاعدة البيانات");

    // 1. تجربة إدارة المهام اليومية (صباح / مهام اليوم / مساء)
    println!("1. تجربة إدارة المهام اليومية (Daily Tasks):");
    let task1 = TaskManager::create_task(
        &storage,
        "إعداد تقرير الميزانية السنويّة 2026",
        Some("مراجعة كشوف الحسابات وتدقيق الأرقام مع الإدارة المالية"),
        Some("morning"),
        Some("finance,ميزانية"),
    )
    .unwrap();

    let task2 = TaskManager::create_task(
        &storage,
        "Build Tauri v2 Desktop Engine with BiDi Support",
        Some("Cross-platform verification on Windows, macOS, and Linux"),
        Some("today"),
        Some("code,rust"),
    )
    .unwrap();

    let task3 = TaskManager::create_task(
        &storage,
        "قراءة ملخص الأبحاث والملاحظات المسائية",
        Some("مراجعة الأفكار المسجلة في صندوق الالتقاط السريع"),
        Some("evening"),
        Some("reading,مساء"),
    )
    .unwrap();

    println!("   ✓ تم إنشاء 3 مهام موزعة على فترات اليوم:");
    println!("     - [صباح] {} (ID: {})", task1.title, task1.id.unwrap());
    println!("     - [اليوم] {} (ID: {})", task2.title, task2.id.unwrap());
    println!("     - [مساء] {} (ID: {})", task3.title, task3.id.unwrap());

    // إنجاز مهمة
    let is_done = TaskManager::toggle_task(&storage, task1.id.unwrap()).unwrap();
    println!(
        "   ✓ تحديث حالة الإنجاز: أصبحت المهمة الأولى مكتملة ({})\n",
        is_done
    );

    // 2. تجربة الالتقاط السريع للملاحظات والروابط (Quick Capture)
    println!("2. تجربة صندوق الالتقاط السريع (Quick Capture):");
    let note1 = CaptureManager::create_capture(
        &storage,
        "مُلاَحَظَةٌ: رابط وثائق معمارية رَوَاق الرسمية",
        Some("link"),
        Some("docs,architecture"),
        Some("https://rawaq.app/docs/architecture"),
    )
    .unwrap();

    let note2 = CaptureManager::create_capture(
        &storage,
        "فكرة تصميم: استخدام الخصائص المنطقية في CSS حصريًا لمنع أخطاء RTL",
        Some("text"),
        Some("design,bidi"),
        None,
    )
    .unwrap();

    println!("   ✓ تم التقاط ملاحظتين وتطهيرهما أمنيًا بنجاح:");
    println!(
        "     - [{}] {} (الرابط: {})",
        note1.note_type,
        note1.content,
        note1.source_url.unwrap_or_default()
    );
    println!("     - [{}] {}\n", note2.note_type, note2.content);

    // 3. تجربة مؤقت التركيز وبومودورو (Focus Timer & Session Persistence)
    println!("3. تجربة مؤقت التركيز واستعادة الجلسة (Focus Timer):");
    let timer = PomodoroTimer::new();

    // بدء جلسة تركيز مرتبطة بالمهمة الثانية لمدة 25 دقيقة (1500 ثانية)
    let status_start = timer.start(task2.id, Some(1500));
    println!(
        "   ✓ بدء جلسة تركيز: الحالة: {:?} | الهدف: {} دقيقة | مرتبطة بالمهمة ID: {:?}",
        status_start.state,
        status_start.target_seconds / 60,
        status_start.task_id
    );

    // محاكاة مرور 5 دقائق (300 ثانية)
    let status_ticked = timer.tick(300);
    println!(
        "   ✓ مرور 5 دقائق من العمل: المنقضي: {} ثانية | المتبقي: {} ثانية",
        status_ticked.elapsed_seconds, status_ticked.remaining_seconds
    );

    // حفظ الجلسة في قاعدة بيانات SQLite
    timer
        .save_to_storage(&storage)
        .expect("فشل حفظ جلسة المؤقت");
    println!("   ✓ تم حفظ حالة الجلسة في قاعدة بيانات SQLite المحلية.");

    // محاكاة إغلاق التطبيق وإعادة فتحه واستعادة الجلسة
    let restored_timer = PomodoroTimer::new();
    let status_restored = restored_timer
        .load_from_storage(&storage)
        .expect("فشل استعادة الجلسة");
    println!("   ✓ تم استعادة الجلسة بعد إعادة تشغيل التطبيق بنجاح!");
    println!(
        "     الحالة المستعادة: {:?} | المنقضي بدقة: {} ثانية | المتبقي: {} ثانية\n",
        status_restored.state, status_restored.elapsed_seconds, status_restored.remaining_seconds
    );
    assert_eq!(status_restored.state, PomodoroState::Running);
    assert_eq!(status_restored.elapsed_seconds, 300);

    // 4. تجربة محرك البحث المحلي الموحّد فائق السرعة مع توحيد الأحرف العربية
    println!("4. تجربة البحث المحلي الموحّد (Unified Local Search):");
    let sample_files = vec![
        SearchableFile {
            name: "مِلَفُّ_أَبْحَاث_الميزانية_2026.pdf".to_string(),
            path: "C:/Desktop/مستندات/ملف_أبحاث_الميزانية_2026.pdf".to_string(),
        },
        SearchableFile {
            name: "Design_System_Rawaq.sketch".to_string(),
            path: "C:/Desktop/Design_System_Rawaq.sketch".to_string(),
        },
        SearchableFile {
            name: "عقد_العمل_المعتمد.pdf".to_string(),
            path: "C:/Desktop/عقد_العمل_المعتمد.pdf".to_string(),
        },
    ];

    // أ) بحث بكلمة عربية مجردة "اعداد تقرير الميزانيه" (همزة عادية وهاء بدلاً من تاء مربوطة)
    let q1 = "اعداد تقرير الميزانيه";
    let resp1 = UnifiedSearchEngine::search(&storage, q1, &sample_files);
    println!("   ✓ استعلام عربي مجرد: '{}':", q1);
    println!(
        "     - استغرق البحث: {} مللي ثانية (أسرع بكثير من شرط 50ms!)",
        resp1.elapsed_ms
    );
    println!(
        "     - عدد النتائج المطابقة عبر المهام والملفات: {}",
        resp1.results.len()
    );
    for item in &resp1.results {
        println!(
            "       • [{:?}] {} — {}",
            item.kind, item.title, item.subtitle
        );
    }
    assert!(resp1.elapsed_ms < 50);
    assert!(!resp1.results.is_empty());

    // ب) بحث عن كلمة "ملاحظه" مع وجود تشكيل كامل في القاعدة "مُلاَحَظَةٌ"
    let q2 = "ملاحظه";
    let resp2 = UnifiedSearchEngine::search(&storage, q2, &sample_files);
    println!("\n   ✓ استعلام بتجاوز التشكيل: '{}' ضد 'مُلاَحَظَةٌ':", q2);
    println!("     - استغرق البحث: {} مللي ثانية", resp2.elapsed_ms);
    for item in &resp2.results {
        println!(
            "       • [{:?}] {} — {}",
            item.kind, item.title, item.subtitle
        );
    }
    assert!(resp2.elapsed_ms < 50);

    // ج) بحث بالإنجليزية مع تجاهل حالة الأحرف "TAURI desktop"
    let q3 = "TAURI desktop";
    let resp3 = UnifiedSearchEngine::search(&storage, q3, &sample_files);
    println!("\n   ✓ استعلام إنجليزي غير حساس لحالة الأحرف: '{}':", q3);
    println!("     - استغرق البحث: {} مللي ثانية", resp3.elapsed_ms);
    for item in &resp3.results {
        println!(
            "       • [{:?}] {} — {}",
            item.kind, item.title, item.subtitle
        );
    }
    assert!(resp3.elapsed_ms < 50);

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  ✅ نجحت جميع تجارب المهام والالتقاط والمؤقت والبحث بنسبة 100%");
    println!("═══════════════════════════════════════════════════════════════");
}
