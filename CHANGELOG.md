# سجل التغييرات (Changelog)

جميع التغييرات الملحوظة في مشروع **رَوَاق (Rawaq)** توثّق في هذا الملف.
يتبع هذا المشروع تنسيق [Keep a Changelog](https://keepachangelog.com/ar/1.0.0/)، ويلتزم بدستور [AGENTS.md](AGENTS.md).

---

## [المرحلة 6] — إدارة المهام اليومية والالتقاط السريع ومؤقت التركيز والبحث الموحد (PR #14 / Issue #13)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **إدارة المهام اليومية المصغرة (`TaskManager`):**
  - نظام مهام سريع موجه حسب الفترات اليومية: الصباح الباكر (`morning`)، مهام اليوم المباشرة (`today`)، والمساء (`evening`).
  - التحقق الأمني من المدخلات عبر `PathSanitizer` لمنع هجمات XSS وتنظيف المسافات والرموز الخبيثة وحظر النصوص الفارغة.
  - إمكانية ربط المهمة بصندوق محدد أو تشغيلها منفردة، مع استعلامات فرز وتحديث وإكمال وحذف آمنة ومباشرة في SQLite.
- **ملاحظات الالتقاط السريع (`CaptureManager`):**
  - تدوين الأفكار والملاحظات الخاطفة في أقل من ثانية وتخزينها في قاعدة البيانات المحلية.
  - دعم التثبيت (`is_pinned`) والتحويل الفوري إلى مهمة عمل بنقرة واحدة، مع تطهير النصوص عبر `PathSanitizer::sanitize_text`.
- **مؤقت بومودورو للتركيز وحماية الجلسات (`PomodoroTimer`):**
  - آلة حالات صارمة للتركيز: خامل (`Idle`) ↔ تركيز (`Focusing`) ↔ استراحة قصيرة (`ShortBreak`) ↔ استراحة طويلة (`LongBreak`) ↔ موقوف مؤقتًا (`Paused`).
  - التخزين الذري لجلسات التركيز في جدول `focus_sessions` الجديد (الترحيل `MIGRATION_V2`).
  - حماية الجلسات من انقطاع التيار الكهربائي أو الانهيار المفاجئ (`Crash Resilience`): إمكانية استعادة آخر حالة نشطة وحساب الفارق الزمني عبر `get_active_focus_session`.
- **محرك البحث الموحد عالي السرعة (`UnifiedSearchEngine`):**
  - محرك بحث متقدم يبحث في جزء من المليثانية (< 50ms) عبر أربعة مصادر في وقت واحد: الملفات، الصناديق الافتراضية، المهام اليومية، وملاحظات الالتقاط السريع.
  - مطبّع النصوص العربية المتقدم (`normalize_arabic_text`):
    - توحيد همزات الألف (`أ`, `إ`, `آ` -> `ا`).
    - توحيد التاء المربوطة والهاء (`ة` -> `ه`).
    - توحيد الألف المقصورة والياء (`ى` -> `ي`).
    - تجريد علامات التشكيل الكاملة (الفتحة، الضمة، الكسرة، التنوين، الشدة، السكون) وحذف التطويل (الكشيدة `ـ`).
    - دعم البحث متعدد الكلمات بأي ترتيب (`multi-term matching`).
- **أوامر التواصل الآمن في Tauri (IPC Commands):**
  - `list_tasks_command`, `create_task_command`, `update_task_command`, `delete_task_command`
  - `list_capture_notes_command`, `create_capture_note_command`, `update_capture_note_command`, `delete_capture_note_command`
  - `start_focus_timer_command`, `pause_focus_timer_command`, `resume_focus_timer_command`, `stop_focus_timer_command`, `get_focus_timer_state_command`
  - `unified_search_command`
- **مكونات الواجهة الرسومية ثنائية الاتجاه (`BiDi UI`):**
  - مكوّن `ProductivityHub`: واجهة تفاعلية شاملة تضم شريط البحث الموحد مع كبسولات تصنيف النتائج وسرعة الاستجابة، تبويبات المهام اليومية المقسمة زمنيًا مع شريط نسبة الإنجاز اليومي، نافذة منبثقة لمؤقت التركيز مع شريط تقدم دائري وأزرار التحكم، وبطاقات الملاحظات السريعة مع ميزة التثبيت والتحويل لمهمة.
  - استخدام حصري للخصائص المنطقية في CSS (`bidi.css`) وتوافقية تامة مع الشاشات العربية (`RTL`) والإنجليزية (`LTR`).
- **الاختبارات الآلية المكتملة:**
  - `productivity::test_pomodoro_state_machine_transitions_and_persistence`
  - `productivity::test_tasks_and_captures_crud_with_edge_inputs`
  - `search::test_unified_search_arabic_normalization_and_speed`
  - `ui::renders ProductivityHub in Arabic (RTL) and filters tasks by period`
  - `ui::toggles task completion and displays progress bar`
  - `ui::handles Quick Capture notes: creation, pinning, and converting to task`
  - `ui::controls Focus Timer: start, pause, resume, and stop`
  - `ui::executes Unified Search and categorizes results`
- **البرنامج التوضيحي الواقعي (`demo_productivity`):**
  - اختبار عملي شامل يثبت إنشاء المهام اليومية، الالتقاط السريع، دورات مؤقت التركيز مع التخزين والاسترجاع عند الانهيار، والبحث فائق السرعة في 0 ميلي ثانية مع التطبيع العربي المتقدم.

---

## [المرحلة 5] — كبسولة الشاشة العلوية التفاعلية والدرج المؤقت والحافظة (PR #12 / Issue #11)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **درج الملفات المؤقت (`QuickShelfManager`):**
  - تجميع مراجع الملفات بالسحب والإفلات مؤقتًا دون نقل أو تعديل أو حذف أي ملف على القرص (`Zero Filesystem Mutations`).
  - التحقق الأمني من المسارات عبر `PathSanitizer`، والتعامل المرن مع الملفات المحذوفة أو المفقودة خارج البرنامج بوسمها «غير متاحة» دون انهيار التطبيق.
  - إزالة مرجع فردي أو إفراغ الدرج المؤقت بالكامل مع بقاء الملفات الأصلية على القرص بنسبة 100%.
- **محرك كشف ملء الشاشة والإخفاء التلقائي (`FullscreenDetector`):**
  - إدارة آلة حالات الكبسولة (`Visible` ↔ `Collapsed` ↔ `HiddenFullscreen`).
  - إخفاء الكبسولة تلقائيًا عند دخول أي تطبيق خارجي أو لعبة أو فيديو وضع ملء الشاشة، واستعادة حالتها السابقة بسلاسة فور الخروج.
- **خدمة إجراءات الحافظة الذكية بموافقة صريحة (`SmartClipboardService`):**
  - احترام خصوصية المستخدم: عدم فحص الحافظة إطلاقًا إلا بموافقة صريحة (`opt_in = true`).
  - التطهير الأمني الصارم والحظر التلقائي للروابط الخبيثة وحقن السكربتات (`javascript:`, `vbscript:`, `data:`, `file://` غير المصرح بها) وتصنيفها `RejectedDangerous`.
  - اكتشاف الروابط الآمنة (`SafeUrl`) وتجريد وسوم HTML من النصوص مع الاحتفاظ بالمحتوى النظيف وحفظها في قاعدة بيانات SQLite المحلية (`capture_notes`).
- **أوامر التواصل الآمن في Tauri (IPC Commands):**
  - `add_to_quick_shelf_command`, `get_quick_shelf_items_command`, `remove_from_quick_shelf_command`, `clear_quick_shelf_command`
  - `set_fullscreen_active_command`, `toggle_capsule_collapse_command`, `get_capsule_state_command`
  - `process_clipboard_command`, `save_clipboard_note_command`
- **مكونات الواجهة الرسومية ثنائية الاتجاه (`BiDi UI`):**
  - مكوّن `TopCapsule`: كبسولة عائمة أعلى الشاشة تدعم وضع الشريط المصغر (`Pill Mode`) ووضع الجزيرة التفاعلية الممتدة (`Living Island`)، وتبويبات الدرج المؤقت وإجراءات الحافظة مع إشعار ملء الشاشة.
  - تصميم ثنائي الاتجاه يعتمد حصريًا على الخصائص المنطقية في CSS (`bidi.css`) دون أي خاصية فيزيائية (`left/right`).
- **الاختبارات الآلية المكتملة:**
  - `island::test_quick_shelf_holds_references_without_moving_source`
  - `island::test_fullscreen_detection_hides_and_restores_capsule`
  - `island::test_clipboard_sanitizer_blocks_malicious_uris`
  - `ui::renders TopCapsule in expanded mode with Quick Shelf in Arabic (RTL)`
  - `ui::renders TopCapsule in collapsed pill mode and toggles expansion`
  - `ui::renders fullscreen auto-hide notice when isFullscreen is active`
  - `ui::handles Smart Clipboard suggestions: actionable URL vs rejected malicious content`
- **البرنامج التوضيحي الواقعي (`demo_island`):**
  - تجربة واقعية لإنشاء ملفات وفحص بصمات `SHA-256` وإثبات عدم المساس بالقرص، كشف ملء الشاشة، وتطهير الحافظة وحفظ الملاحظات في SQLite.

---

## [المرحلة 4] — محرك الفرز الآمن والتراجع بضغطة زر (PR #10 / Issue #9)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **محرك الفرز الآمن للنواة الخلفية (`SafeOrganizerEngine`):**
  - فحص وحساب خطة المعاينة الإلزامية (`Dry-Run Preview`) مع ضمان الصفرية المطلقة للتعديلات على القرص أثناء المعاينة (`Zero Filesystem Mutations`).
  - كشف وحل تعارض أسماء الملفات تلقائيًا (`Collision Avoidance`) بإضافة لواحق رقمية آمنة مثل ` (1)` و` (2)` مع المنع البات للكتابة فوق أي ملف قائم.
  - تطبيق بروتوكول النقل الآمن الذري المعتمد في `ADR-0002`:
    1. تسجيل العملية بحالة `PENDING` في جدول `undo_operations`.
    2. نسخ الملف إلى الوجهة.
    3. التحقق الرياضي الصارم من تطابق بصمة `SHA-256` وحجم الملف.
    4. في حال الفشل أو اختلاف البصمة: حذف النسخة التالفة في الوجهة فورًا، إبقاء المصدر سليمًا 100%، وتسجيل حالة `FAILED`.
    5. في حال التطابق: حذف الملف من المصدر، وتحديث سجل التراجع إلى `COMPLETED`.
  - معالجة ذكية ومفصلة للأخطاء وحالات الملفات المقفولة أو المحمية (`Locked / Permission Denied`) دون إيقاف بقية الدفعة.
  - محرك التراجع الكامل بنقرة زر (`One-Click Undo Engine`) القادر على استرجاع دفعات النقل كاملة بدقة متناهية إلى مساراتها الأصلية مع التحقق من سلامة البصمات.
- **أوامر التواصل الآمن في Tauri (IPC Commands):**
  - `preview_organization_command`: حساب خطة المعاينة وقوائم النقل والاستبعاد.
  - `execute_organization_command`: تنفيذ الفرز الآمن الذري للدفعة وتوليد تقرير تفصيلي.
  - `undo_organization_command`: التراجع عن دفعة فرز سابقة واستعادة الملفات لمساراتها الأصلية.
  - `list_undo_batches_command`: استعراض الدفعات السابقة وسجل العمليات.
- **مكونات الواجهة ثنائية الاتجاه (`BiDi UI`):**
  - مكوّن `OrganizerPreview`: جدول معاينة تفاعلي يعرض المسارات الأصلية والوجهات المقترحة مع شارات الأمان لكشف التعارضات والعناصر المتخطاة، وزر تراجع فوري (`Undo`) بعد التنفيذ.
  - أنماط بصرية ثنائية الاتجاه بخصائص CSS المنطقية حصريًا (`bidi.css`) دون أي خاصية فيزيائية (`left/right`).
- **الاختبارات الآلية المكتملة:**
  - `organizer::test_dry_run_zero_filesystem_mutations`
  - `organizer::test_collision_never_overwrites_existing_file`
  - `organizer::test_checksum_mismatch_aborts_and_preserves_source`
  - `organizer::test_locked_or_permission_denied_file_reported_clearly`
  - `organizer::test_undo_batch_restores_exact_original_paths`
  - `ui::renders Dry-Run preview with collision detection in Arabic (RTL)`
  - `ui::calls execute and undo flows correctly`
  - `ui::handles empty rules and displays skipped items note`
- **البرنامج التوضيحي الواقعي (`demo_organizer`):**
  - تشغيل مسار تنظيمي كامل على ملفات حقيقية بأسماء عربية وإنجليزية وحساب بصمات `SHA-256`، إثبات عدم التعديل أثناء المعاينة، تفادي التعارضات، والتراجع بنقرة زر واحدة.

---

## [المرحلة 3] — صناديق تنظيم سطح المكتب ووضع الكبسولة والمجموعات (PR #8 / Issue #7)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **خدمة الصناديق الافتراضية في النواة الخلفية (`BoxService`):**
  - إنشاء وتعديل وحذف الصناديق في قاعدة البيانات مع الالتزام الصارم بميثاق أمان الملفات: عدم تعديل أو حذف أو نقل أي ملف على قرص المستخدم أبدًا.
  - فحص محتويات المجلدات بأمان وتصنيف العناصر (`BoxItem`) مع كشف نوع الملف، الحجم، وتاريخ التعديل.
  - فرز محتويات الصناديق حسب الاسم، النوع (المجلدات أولًا ثم الامتداد)، التاريخ (الأحدث)، والحجم (الأكبر).
  - كشف حالة المجلدات غير المتاحة أو الأقراص المفصولة (`MissingOrDisconnected`) وإرجاع حالة آمنة تحافظ على وجود الصندوق وإعداداته في قاعدة البيانات دون انهيار التطبيق.
  - دمج عدة صناديق في مجموعة تبويبات واحدة (`BoxGroup`) مع الحفاظ على إعدادات كل صندوق وفك المجموعات بأمان تام.
- **مكونات الواجهة الرسومية ثنائية الاتجاه (`BiDi UI`):**
  - مكوّن `DesktopBox`: يدعم وضع الشبكة (`Grid`) ووضع القائمة (`List`)، ومحاذاة أسماء الملفات المختلطة بعزل اتجاه عبر `<BdiText>`.
  - وضع الكبسولة (`Capsule Mode`): طيّ الصندوق إلى شريط كبسولي أفقي مصغّر وتمدده بسلاسة بنقرة زر في الاتجاهين العربي (`RTL`) والإنجليزي (`LTR`).
  - مكوّن `BoxGroupTabs`: عرض تبويبات للتنقل بين الصناديق المدمجة وفك المجموعة.
  - راية تحذير آمنة وأنيقة عند فصل المجلد المربوط أو غيابه.
- **أوامر التواصل الآمن (Tauri IPC):**
  - `scan_box_contents_command`
  - `create_box_command`
  - `update_box_command`
  - `delete_box_command`
  - `merge_boxes_command`
  - `dissolve_group_command`
- **الاختبارات الآلية المكتملة:**
  - `boxes::test_map_folder_never_modifies_or_deletes_disk_files`
  - `boxes::test_missing_or_disconnected_folder_graceful_state`
  - `boxes::test_group_merge_and_dissolve_preserves_members`
  - `boxes::test_sorting_arabic_english_and_directories`
  - `ui::test_capsule_collapse_expand_and_rtl_layout`

---

## [المرحلة 2] — طبقة التخزين المحلي في SQLite والنسخ الاحتياطي الآمن (PR #6 / Issue #5)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **محرك قاعدة البيانات المحلية `storage-sqlite`:**
  - تفعيل وضع `WAL` (Write-Ahead Logging) لضمان متانة العمليات والتعامل المتزامن دون إقفال كامل للقاعدة.
  - محرك ترقيات الجداول التلقائي والحتمي (`Idempotent Migrations`) للإصدار V1.
- **جداول حفظ البيانات المحلية:**
  - `desktop_boxes`: حفظ مواقع وأبعاد وإعدادات الصناديق وربطها بهوية الشاشات والمجموعات.
  - `box_groups`: مجموعات الصناديق للتبويب والتنظيم المشترك.
  - `organize_rules`: قواعد تصنيف الملفات والامتدادات والمجلدات الهدف.
  - `task_items`: إدارة المهام اليومية (صباح / اليوم / مساء) وحالة الإنجاز والوسوم.
  - `capture_notes`: الملاحظات والروابط السريعة الملتقطة مع نوع المحتوى والوسوم.
  - `undo_operations`: سجل تفصيلي لعمليات النقل التلقائي مع البصمة والحجم والحالة لضمان التراجع الذري.
- **محرك النسخ الاحتياطي الآمن (`Backup & Restore`):**
  - تصدير كامل بيانات البرنامج محليًا إلى ملف JSON موثّق ببصمة تدقيق `SHA-256`.
  - استيراد النسخ الاحتياطية مع التحقق الصارم من تطابق البصمة قبل لمس قاعدة البيانات، ورفض الحزم التالفة أو المتلاعب بها.
- **الاختبارات الآلية:**
  - `storage::test_migrations_idempotent_and_clean`
  - `storage::test_concurrent_reads_and_atomic_rollback_on_error`
  - `storage::test_arabic_and_emoji_unicode_roundtrip`
  - `backup::test_rejects_corrupted_or_tampered_backup_archive`
  - تم التحقق والنجاح على Windows و macOS و Linux في GitHub Actions.

---

## [المرحلة 1] — الهيكل البرمجي الأساسي، الأمان، وتعدد اللغات (PR #4 / Issue #2)
*التاريخ: 2026-11-12*

### المضاف (Added)
- **الهيكل البرمجي الأساسي لـ Tauri v2:**
  - نواة خلفية بلغة `Rust` مع تقييد الصلاحيات (`capabilities/default.json`).
  - واجهة أمامية بـ `React 19` و`TypeScript` و`Vite`.
  - أيقونات التطبيق الأصلية بجميع المقاسات لنظام ويندوز وماك ولينكس (`ico`, `icns`, `png`).
- **بوابة الأمان وتنظيف المدخلات (`CommandGate` & `PathSanitizer`):**
  - فحص أمني للمسارات ورفض محاولات عبور المسارات (`Path Traversal` مثل `../` و`%2e%2e`)، والرموز الصفرية (`Null Bytes`)، ومسارات الشبكة (`UNC`)، والأسماء المحجوزة في ويندوز (`CON`, `NUL`)، ومسارات النظام الحساسة (`/etc`, `C:\Windows\System32`).
  - تطهير وتجريد النصوص والروابط من الشيفرات الخبيثة وحقن السكربتات (`XSS`).
- **نظام اللغات ثنائي الاتجاه الأصلي (`BiDi i18n Engine`):**
  - كتالوجات ترجمة متكاملة باللغتين العربية (`ar.json`) والإنجليزية (`en.json`).
  - فحص تطابق مفاتيح الترجمة والمتغيرات (`Placeholders`) آليًا بنسبة 100% في كل من Rust وTypeScript.
  - تبديل فوري للغة واتجاه الواجهة (`dir="rtl"` ↔ `dir="ltr"`) في وقت التشغيل دون إعادة تشغيل البرنامج.
  - عزل أسماء الملفات والمسارات المختلطة عبر مكوّن `<bdi dir="auto">`.
  - اعتماد الخصائص المنطقية حصريًا في CSS لمنع انكسار الواجهة عند التبديل.

---

## [المرحلة 0] — ملفات الأساس والدستور المعماري (PR #3 / Issue #1)
*التاريخ: 2026-11-12*

### المضاف (Added)
- وثيقة متطلبات المنتج المعتمدة (`docs/PRD.md`).
- المعمارية الهندسية الشاملة (`docs/ARCHITECTURE.md`).
- خارطة الطريق المرقّمة وشروط النجاح (`docs/ROADMAP.md`).
- سجل القرارات المعمارية الخمسة الكبرى (`docs/adr/0001` إلى `0005`).
- دستور وقواعد عمل الوكلاء والمطورين (`AGENTS.md`).
- خط أنابيب التكامل المستمر على الأنظمة الثلاثة (`.github/workflows/ci.yml`).
- سير عمل بناء ونشر الإصدارات الآلي من GitHub فقط (`.github/workflows/release.yml`).
- قالب الـ Pull Request الإلزامي (`.github/PULL_REQUEST_TEMPLATE.md`).
