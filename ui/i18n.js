/* Russian source strings with an English UI dictionary. User content is left intact. */
window.KyroI18n=(()=>{
 const dictionary=Object.fromEntries(`
Палитра цветов|Color palette
Ошибка|Error
ID зарегистрированного приложения|Registered application ID
Microsoft подключён. Можно запускать Minecraft.|Microsoft connected. You can launch Minecraft.
Откройте|Open
в браузере и введите код:|in your browser and enter the code:
Быстрая отрисовка мира|Faster world rendering
Быстрая отрисовка для Forge|Faster rendering for Forge
Оптимизация игровой логики|Game logic optimization
Меньше расход памяти|Lower memory usage
Быстрая отрисовка интерфейса и объектов|Faster UI and entity rendering
Не рисует скрытые объекты|Skips rendering hidden objects
Оптимизация загрузки и памяти|Loading and memory optimization
Оптимизация отрисовки и использования памяти|Rendering and memory optimization
Уже выбран другой мод отрисовки|Another rendering mod is already selected
Уже установлен (включая отключённые)|Already installed (including disabled mods)
Нет версии для этого Minecraft и ядра|No version for this Minecraft and loader
Для FPS-буста выбери Fabric, Quilt, Forge или NeoForge. Vanilla не загружает моды.|Choose Fabric, Quilt, Forge or NeoForge for FPS Boost. Vanilla does not load mods.
Нужная зависимость отключена. Включи её в менеджере модов.|A required dependency is disabled. Enable it in the mod manager.
Зависимость|Dependency
Профиль изменился. Открой FPS-буст снова.|The profile changed. Open FPS Boost again.
Выбран другой профиль. Повтори подбор модов.|A different profile is selected. Find compatible mods again.
Список модов изменился. Повтори подбор перед установкой.|The mod list changed. Find compatible mods again before installing.
FPS-буст установлен|FPS Boost installed
Не хватило памяти|Not enough memory
Java сообщила об ошибке выделения памяти. Проверь объём RAM для игры; если резервирование не удалось, уменьши его и закрой другие приложения.|Java reported a memory allocation error. Check the RAM allocation; if reservation failed, reduce it and close other applications.
Неподходящая Java|Incompatible Java version
Клиенту или моду нужна другая версия Java. Включи автоматический подбор и повтори запуск.|The game or a mod needs a different Java version. Enable automatic selection and try again.
Не совпали моды или зависимости|Mod or dependency mismatch
В сообщении загрузчика есть несовместимые версии или отсутствующие зависимости. Проверь состав модов текущего профиля.|The loader reported incompatible versions or missing dependencies. Check this profile's mods.
Ошибка применения Mixin|Mixin application error
Ошибка графической системы|Graphics system error
Minecraft не смог создать графическое окно. Проверь драйвер видеокарты и поддержку требуемой версии OpenGL.|Minecraft could not create a graphics window. Check your graphics driver and the required OpenGL support.
Не удалось загрузить файлы клиента|Unable to load game files
Есть признак отсутствующего или повреждённого файла. Запусти проверку Minecraft; если ошибка относится к моду, переустанови его.|A file may be missing or corrupted. Verify Minecraft files; if a mod is affected, reinstall it.
Запуск отменён пользователем|Launch cancelled by user
Игра или подготовка ещё выполняется|Game or preparation is still running
Последний запуск завершился штатно|The last session ended normally
Пока нет данных о запуске|No launch data yet
Найдены признаки причины вылета|Possible crash causes found
Причина автоматически не определена|Unable to identify the cause automatically
Выводы основаны на сигнатурах локального лога. Если явной причины нет, открой полный лог.|Findings are based on local log patterns. If the cause is unclear, open the full log.
Локальная диагностика|Local diagnostics
Java сообщила об ошибке выделения памяти.|Java reported a memory allocation error.
Другое ядро|Different mod loader
Повторяющийся мод|Duplicate mod
Несовместимые моды|Incompatible mods
Не подходит версия зависимости|Incompatible dependency version
Версия требует проверки|Version needs checking
Не хватает зависимости|Missing dependency
Манифест не распознан|Unrecognized manifest
Проверить совместимость этого JAR автоматически не удалось.|Could not automatically check this JAR's compatibility.
Проверка локальных манифестов и вложенных модов. Игровые конфликты могут проявиться при запуске.|Checks local manifests and nested mods. In-game conflicts may still occur at launch.
Играть|Play
Каталог|Catalog
Скриншоты|Screenshots
Диагностика|Diagnostics
Активность|Activity
Уведомления|Notifications
Настройки|Settings
ЯЗЫК|LANGUAGE
Язык интерфейса лаунчера.|Launcher interface language.
ОФОРМЛЕНИЕ|APPEARANCE
Твой стиль. Ничего лишнего.|Your style. Nothing extra.
Тема|Theme
Палитра|Palette
Тёмная|Dark
Полночь|Midnight
Чёрная|Black
Светлая|Light
Лаванда|Lavender
Синий|Blue
Бирюзовый|Teal
Зелёный|Green
Золотистый|Gold
Коралловый|Coral
Розовый|Pink
Фиолетовый|Purple
Серебристый|Silver
ПРИ ЗАПУСКЕ ИГРЫ|WHEN THE GAME STARTS
Что делать с окном лаунчера.|Choose what happens to the launcher window.
Скрыть|Hide
Свернуть|Minimize
Оставить открытым|Keep open
После закрытия игры лаунчер вернётся.|The launcher returns when you close the game.
Показывать активность Minecraft.|Share your Minecraft activity.
Активность Discord|Discord activity
Показывать адрес сервера|Show server address
УЧЁТНАЯ ЗАПИСЬ|ACCOUNT
УЧЁТНЫЕ ЗАПИСИ|ACCOUNTS
Аккаунты|Accounts
Аккаунт не выбран|No account selected
Добавить аккаунт|Add account
Управлять|Manage
Не выбран|Not selected
Офлайн-аккаунт|Offline account
Офлайн / Microsoft|Offline / Microsoft
ОФОРМЛЕНИЕ ЛАУНЧЕРА|LAUNCHER APPEARANCE
Выбранный фон отображается во всём окне и на всех вкладках.|Your background appears throughout the launcher.
Орбита|Orbit
Карточка|Card
Загрузить фон|Upload background
Убрать фон|Remove background
Убрать обложку|Remove cover
PNG, JPEG или WebP до 4 МБ. Файлы хранятся локально.|PNG, JPEG or WebP up to 4 MB. Files are stored locally.
ОБНОВЛЕНИЯ KYRO|KYRO UPDATES
Текущая версия|Current version
Настройки и игровые профили сохраняются.|Settings and game profiles are preserved.
Проверить обновления|Check for updates
Проверяем GitHub…|Checking GitHub…
Установлена актуальная версия|You're up to date
Обновить и перезапустить|Update and restart
Автоматическое обновление пока недоступно.|Automatic updates are not available yet.
Основные|General
Java и память|Java & memory
Загрузки|Downloads
Профили графики|Graphics presets
Изменения сохраняются автоматически|Changes are saved automatically
ПАМЯТЬ|MEMORY
Память в ГБ|Memory in GB
ГБ выделено для игры.|GB allocated to the game.
Выделяй не больше половины доступной RAM.|Allocate no more than half of your available RAM.
Minecraft 1.21 требует Java 21 x64.|Minecraft 1.21 requires Java 21 x64.
Путь к Java|Java path
Можно указать полный путь к java.exe или команду из PATH.|Enter the full path to java.exe or a command from PATH.
АРГУМЕНТЫ JVM|JVM ARGUMENTS
Аргументы JVM|JVM arguments
Передаются Java напрямую, без командной оболочки.|Passed directly to Java without a command shell.
Сбросить к рекомендуемым|Reset to recommended
АВТОМАТИЧЕСКАЯ JAVA|AUTOMATIC JAVA
Подходящая Temurin загружается в данные KYRO, без системной установки.|A compatible Temurin runtime is downloaded to KYRO data without a system-wide installation.
Подбирать Java для Minecraft|Select Java automatically
Скачать Java для выбранной версии|Download Java for this version
ЗАГРУЗКИ|DOWNLOADS
Параллельная загрузка файлов.|Download files in parallel.
Потоков:|Threads:
Потоки загрузки|Download threads
ФАЙЛЫ И ЦЕЛОСТНОСТЬ|FILES & INTEGRITY
Проверка по SHA-1 при загрузке Minecraft и модов.|SHA-1 verification when downloading Minecraft and mods.
Перепроверить файлы Minecraft|Verify Minecraft files
Папка профиля|Profile folder
Папка лаунчера|Launcher folder
Данные лаунчера|Launcher data
Каталог с EXE|Executable directory
Каталог данных|Data directory
Лог игры|Game log
Лог установки ядра|Loader installation log
Папка профиля содержит моды, миры и настройки выбранной версии и ядра. Папка данных содержит общий кеш и настройки KYRO.|The profile folder contains mods, worlds and settings for this version and loader. The data folder contains shared cache and KYRO settings.
Максимум FPS|Maximum FPS
Красиво|Quality
Для слабого ПК|Low-end PC
Больше ресурсов на частоту кадров.|Prioritize a higher frame rate.
Дальняя прорисовка и плавные границы текстур.|Longer render distance and smoother textures.
Меньше нагрузка на графику и процессор.|Reduce GPU and CPU usage.
Только текущий профиль:|Current profile only:
Меняются ванильные параметры в options.txt. Клавиши, звук и остальные настройки сохраняются. Перед применением закрой игру; исходный файл сохраняется в kyro-backups.|Changes vanilla settings in options.txt. Key bindings, audio and other settings are preserved. Close the game before applying. The original file is saved in kyro-backups.
Применить|Apply
Откатить последнее применение|Undo last preset
чанков прорисовки|chunk render distance
чанка прорисовки|chunk render distance
симуляция|simulation distance
быстрая графика|fast graphics
детальная графика и облака|fancy graphics and clouds
минимум частиц|minimal particles
все частицы|all particles
сущности|entities
без облаков и VSync|clouds and VSync off
без VSync|VSync off
без ограничения FPS|unlimited FPS
лимит|limit
Проекты для твоего Minecraft|Projects for your Minecraft
Моды|Mods
Шейдеры|Shaders
Ресурспаки|Resource packs
Сборки|Modpacks
Установленные|Installed
Найти в каталоге|Search the catalog
Поиск в каталоге|Search the catalog
Поиск при вводе|Search as you type
Версия Minecraft в каталоге|Catalog Minecraft version
Загрузчик в каталоге|Catalog mod loader
Сборки устанавливаются в отдельные профили. Клиентские необязательные файлы включены; серверные исключены. Поддерживаются Fabric, Quilt, Forge, NeoForge и Vanilla.|Modpacks use separate profiles. Optional client files are included; server files are excluded. Fabric, Quilt, Forge, NeoForge and Vanilla are supported.
Ничего не найдено|No results found
Попробуй другой запрос или версию Minecraft.|Try another search or Minecraft version.
Ищем проекты…|Searching projects…
Не удалось загрузить каталог|Unable to load catalog
Загрузка совместимых проектов.|Loading compatible projects.
Повторить|Retry
Выбрать профиль|Choose profile
Твой игровой профиль|Your game profile
Всё установленное — в одном месте.|Everything installed, in one place.
Всё|All
Все|All
Совместимость|Compatibility
Установлено|Installed
Ресурспак|Resource pack
Сборка|Modpack
Моды не установлены|No mods installed
Обновлений нет|No updates available
Обновления модов|Mod updates
Проверим после игры|We'll check after the game
Проверяем…|Checking…
Не удалось проверить|Unable to check
Открыть менеджер|Open manager
Обновления модов текущего профиля — открыть менеджер|Current profile mod updates — open manager
Обновления модов — открыть менеджер|Mod updates — open manager
Подробнее|Details
Зависимости:|Dependencies:
Локальный мод|Local mod
Включён|Enabled
Выключен|Disabled
Не включён|Not enabled
Включить|Enable
Обновить|Refresh
Здесь начинается твоя сборка|Your modpack starts here
Добавь первый проект из каталога.|Add your first project from the catalog.
Чтение модов…|Reading mods…
Читаем установленные моды…|Reading installed mods…
Загрузки, обновления и сообщения лаунчера|Downloads, updates and launcher messages
Непрочитанные|Unread
Прочитать все|Mark all as read
Очистить|Clear
Удалить уведомление|Dismiss notification
Обновить KYRO|Update KYRO
Эта версия уже установлена. Повторное обновление не требуется.|This version is already installed. No update is needed.
Уже установлено|Already installed
Всё прочитано|All caught up
Пока ничего нового|Nothing new yet
Непрочитанные уведомления появятся здесь.|Unread notifications will appear here.
Сообщим, когда появятся обновления или завершатся загрузки.|We'll let you know about updates and completed downloads.
МОЖНО ВОЗВРАЩАТЬСЯ К ИГРЕ|BACK TO YOUR GAME
Твоя история Minecraft|Your Minecraft history
ВСЕГО В ИГРЕ|TOTAL PLAYTIME
ЭТА НЕДЕЛЯ|THIS WEEK
СЕРИЯ|STREAK
РЕКОРД СЕССИИ|LONGEST SESSION
ЛЮБИМЫЕ ВЕРСИИ|FAVORITE VERSIONS
ОБ УЧЁТЕ АКТИВНОСТИ|ABOUT ACTIVITY TRACKING
ПОСЛЕДНИЕ СЕССИИ|RECENT SESSIONS
Каждая клетка — один день. Наведи курсор: время и число сессий.|Each square is one day. Hover to see playtime and sessions.
Меньше|Less
Больше|More
Здесь появятся версии, в которых ты играешь чаще всего.|Your most played versions will appear here.
Учитываются запуски из KYRO: от старта Java до закрытия игры. Время хранится локально на этом компьютере. Статистика по отдельным серверам требует игрового мода и пока не собирается.|Playtime is tracked from Java starting until the game closes, for games launched by KYRO. Data is stored locally. Per-server statistics require an in-game mod and are not collected yet.
История начинается здесь|Your story starts here
Запусти Minecraft из KYRO — первая сессия появится после закрытия игры.|Launch Minecraft from KYRO. Your first session appears after you close the game.
В этот день не играли|No playtime on this day
Этот день ещё впереди|This day is still ahead
Сессий:|Sessions:
сессий:|sessions:
СЕРВЕРЫ|SERVERS
Серверы|Servers
Избранные серверы|Favorite servers
Добавить сервер|Add server
Добавить любимый сервер|Add a favorite server
Название|Name
Адрес|Address
Мой сервер|My server
Добавить|Add
Отмена|Cancel
Удалить сервер|Remove server
Иконка сервера|Server icon
Нет ответа|No response
Онлайн|Online
Сервер недоступен|Server unavailable
Статус ещё не проверен|Status not checked yet
Подключиться|Connect
Обновить статусы|Refresh status
Избранное · запуск в текущем игровом профиле|Favorites · launch using the current profile
Профиль для подключения:|Connection profile:
Для адреса с нестандартным портом укажи его после двоеточия.|For a custom port, add it after a colon in the address.
Добавь первый сервер|Add your first server
Здесь будут онлайн, задержка и версия. Кнопка подключения запустит выбранный профиль Minecraft.|See player counts, latency and version here. Connect launches the selected Minecraft profile.
Версии и ядра|Versions & loaders
ТВОЙ MINECRAFT|YOUR MINECRAFT
Поиск версии или ядра|Search version or loader
Скачанные|Downloaded
Подготовить выбранную версию|Prepare selected version
Скачать и проверить выбранную версию|Download and verify selected version
Закрыть версии|Close versions
Версия не найдена|Version not found
Пока нет скачанных версий|No downloaded versions yet
Выбери версию и нажми +, чтобы подготовить файлы.|Choose a version and press + to prepare its files.
Попробуй другой номер версии или название ядра.|Try another version number or loader name.
Обновляем список…|Refreshing list…
Версий:|Versions:
Вариантов:|Variants:
Файлы подготовлены|Files prepared
Выбрать версию Minecraft и ядро|Choose Minecraft version and loader
Изменить версию и ядро|Change version and loader
Игровой профиль|Game profile
Обычный профиль|Default profile
Готов к запуску|Ready to play
ГОТОВ К ЗАПУСКУ|READY TO PLAY
ВЫПОЛНЯЕТСЯ ОПЕРАЦИЯ|OPERATION IN PROGRESS
ЗАПУСК…|LAUNCHING…
Подготавливаем игру…|Preparing game…
Добавь аккаунт, чтобы начать игру|Add an account to start playing
Открыть папку текущего профиля|Open current profile folder
Настроить оптимизацию Minecraft|Configure Minecraft optimization
ОПТИМИЗАЦИЯ|OPTIMIZATION
FPS-буст|FPS Boost
Закрыть FPS-буст|Close FPS Boost
Моды для плавной игры. Твои настройки графики сохранятся.|Mods for smoother gameplay. Your graphics settings are preserved.
Подбираем моды|Finding compatible mods
Проверяем совместимость и зависимости…|Checking compatibility and dependencies…
Только совместимые версии|Compatible versions only
ПОДОБРАННЫЕ МОДЫ|SELECTED MODS
К установке:|To install:
Моды оптимизации|Optimization mods
Будет установлен|Will be installed
Уже установлен|Already installed
С зависимостями|Including dependencies
Всё готово|All ready
Установить|Install
Не удалось подобрать моды|Unable to find compatible mods
Твой Minecraft.|Your Minecraft.
Добавь офлайн-аккаунт по нику или войди через Microsoft.|Add an offline account by username or sign in with Microsoft.
Добавить офлайн-аккаунт|Add offline account
Ник офлайн-аккаунта|Offline username
Например, Nova|For example, Nova
От 3 до 16 латинских букв, цифр или _|3–16 Latin letters, numbers or _
Без пароля и Microsoft. Для серверов с проверкой лицензии нужен Microsoft-аккаунт.|No password or Microsoft required. Servers with license verification require a Microsoft account.
Войти через Microsoft|Sign in with Microsoft
Войти в Microsoft|Sign in to Microsoft
Выйти из Microsoft|Sign out of Microsoft
Открыть Microsoft|Open Microsoft
Ожидаем подтверждение…|Waiting for confirmation…
Нужен Client ID приложения с public client flow и доступом Minecraft API. Пароль вводится только на сайте Microsoft.|Requires an application Client ID with public client flow and Minecraft API access. Enter your password only on Microsoft's website.
Срок кода истёк. Попробуйте снова.|The code expired. Please try again.
Офлайн-аккаунт добавлен и выбран.|Offline account added and selected.
Вы вышли из Microsoft.|You signed out of Microsoft.
Удалить|Remove
Выбрать|Select
Выбор|Selection
Закрыть|Close
Развернуть|Maximize
Отменить|Cancel
Скрыть прогресс|Hide progress
Прогресс операции|Operation progress
Обложка:|Cover:
Только текущий профиль:|Current profile only:
Папка|Folder
Открыть папку|Open folder
Лог этого профиля|Profile log
Папка диагностики|Diagnostics folder
Закрыть просмотр|Close preview
Скриншотов пока нет|No screenshots yet
Сделай скриншот клавишей F2 в Minecraft. Он появится в галерее выбранного профиля.|Press F2 in Minecraft to take a screenshot. It appears in this profile's gallery.
Читаем папку screenshots…|Reading screenshots folder…
Не удалось открыть PNG|Unable to open PNG
Проверить|Check
Есть явные несовместимости|Incompatibilities detected
Явных несовместимостей не найдено|No obvious incompatibilities found
Проверено JAR:|JARs checked:
Ошибок:|Errors:
Предупреждений:|Warnings:
Нажми «Проверить». Интернет для проверки локальных манифестов не нужен.|Click Check. Local manifest checks do not require internet access.
Читаем манифесты модов и зависимости…|Reading mod manifests and dependencies…
Читаем результаты последнего запуска…|Reading the latest launch results…
Строки с ошибками|Error details
Проверь моды перед запуском|Check your mods before launching
Найдены возможные проблемы. Можно вернуться к модам или продолжить запуск.|Potential issues found. Return to mods or continue launching.
Открыть проверку|Open compatibility check
Запустить всё равно|Launch anyway
Открыть моды|Open mods
Проверить совместимость|Check compatibility
Перепроверить файлы|Verify files
Открыть лог|Open log
Проверка не завершена|Check incomplete
Подготовка не завершена|Preparation incomplete
Готово.|Done.
Открыто:|Opened:
Дождитесь завершения текущей операции.|Wait for the current operation to finish.
Дождитесь завершения текущей операции|Wait for the current operation to finish
Дождитесь завершения игры или установки.|Wait for the game or installation to finish.
Дождитесь завершения игры или установки, чтобы сохранить историю сессии.|Wait for the game or installation to finish so session history can be saved.
Запуск отменён.|Launch cancelled.
Запуск отменён|Launch cancelled
Отменяем подготовку…|Cancelling preparation…
Отмена подготовки сейчас недоступна.|Preparation cannot be cancelled right now.
Подготовка Minecraft…|Preparing Minecraft…
Подготовка версии…|Preparing version…
Подготовка выбранной версии…|Preparing selected version…
Подготовка автоматической Java…|Preparing automatic Java…
Установка FPS-буста…|Installing FPS Boost…
Запуск сборки…|Launching modpack…
Удаление мода…|Removing mod…
Переключение мода…|Toggling mod…
Обновление мода и зависимостей…|Updating mod and dependencies…
Обновление KYRO…|Updating KYRO…
Проверка Minecraft…|Verifying Minecraft…
Проверка файлов…|Verifying files…
Применение профиля графики…|Applying graphics preset…
Восстановление графики…|Restoring graphics settings…
Подключение к серверу…|Connecting to server…
Minecraft запущен. Можно запустить ещё один экземпляр.|Minecraft launched. You can launch another instance.
Мод удалён из профиля.|Mod removed from the profile.
Сборка установлена и выбрана. Нажми «Играть» для запуска.|Modpack installed and selected. Click Play to launch it.
Файлы проверены. Версия доступна в «Скачанных».|Files verified. The version is available under Downloaded.
Список не обновлён. Показаны сохранённые версии.|Could not refresh the list. Showing saved versions.
Выбери PNG, JPEG или WebP до 4 МБ.|Choose a PNG, JPEG or WebP up to 4 MB.
Не удалось сохранить выбор|Unable to save selection
Не удалось сохранить настройки|Unable to save settings
Эту версию пока нельзя установить автоматически. Попробуй позже.|This version cannot be installed automatically yet. Try again later.
Установлена актуальная версия KYRO.|KYRO is up to date.
Часть каталога недоступна.|Part of the catalog is unavailable.
Предпросмотр интерфейса · запустите EXE для доступа к Rust, Minecraft и Modrinth.|Interface preview · run the EXE to access Minecraft and Modrinth.
Откройте приложение KYRO. В браузере доступен только предпросмотр интерфейса.|Open the KYRO application. The browser only provides an interface preview.
янв|Jan
фев|Feb
мар|Mar
апр|Apr
май|May
июн|Jun
июл|Jul
авг|Aug
сен|Sep
окт|Oct
ноя|Nov
дек|Dec
Пн|Mon
Ср|Wed
Пт|Fri
`.trim().split('\n').map(line=>line.split('|')));
 for(const [ru,en] of Object.entries(dictionary)){if(!dictionary[ru.toUpperCase()])dictionary[ru.toUpperCase()]=en.toUpperCase();}
 const keys=Object.keys(dictionary).sort((a,b)=>b.length-a.length);
 const pattern=new RegExp(keys.map(k=>k.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')).join('|'),'gu');
 let language='ru',observer;const originals=new WeakMap(),attrs=new WeakMap();
 function translate(value){if(language!=='en')return value;let result=value.replace(pattern,(match,offset,whole)=>{const before=whole[offset-1]||'',after=whole[offset+match.length]||'';return /[А-Яа-яЁё]/.test(before)||/[А-Яа-яЁё]/.test(after)?match:dictionary[match];});
  return result.replace(/\b(\d+)\s+ГБ/g,'$1 GB').replace(/(\d+)\s+МБ/g,'$1 MB').replace(/(\d+)\s+мин/g,'$1 min').replace(/(\d+)\s+ч(?=\s|$)/g,'$1 h').replace(/(\d+)\s+дн\./g,'$1 d').replace(/(\d+)\s+мс/g,'$1 ms').replace(/(\d+)\s+онлайн/g,'$1 online').replace(/(\d+)\s+проектов/g,'$1 projects').replace(/(\d+)\s+загрузок/g,'$1 downloads').replace(/(\d+)\s+я(?:дро|дра|дер)/g,'$1 loaders').replace(/Доступно (\d+) обновлени(?:е|я|й)/g,(_,n)=>n+' '+(n==='1'?'update':'updates')+' available').replace(/Доступна /g,'Available: ').replace(/установлено/g,'installed').replace(/файлов/g,'files').replace(/МБ\/с/g,'MB/s').replace(/включён/g,'enabled').replace(/Играть на /g,'Play on ').replace(/Подключение к /g,'Connecting to ').replace(/Установка /g,'Installing ').replace(/Иконка /g,'Icon: ').replace(/Не удалось обновить список версий:/g,'Unable to refresh versions:');
 }
 const protectedSelector='script,style,textarea,input,pre,code,[data-i18n-ignore],#account-name,.account-row strong,.avatar,.server-connect strong,.mod-info strong,.mod-info>p,.mod-details p,.version-display,.version-row-id,.fps-profile,.session-profile,.code-input';
 function apply(){if(!document.body)return;observer?.disconnect();const walker=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT);let node;
  while(node=walker.nextNode()){if(node.parentElement?.closest(protectedSelector))continue;const old=originals.get(node),current=node.nodeValue;const source=old&&current===old.rendered?old.source:current;const rendered=translate(source);originals.set(node,{source,rendered});if(current!==rendered)node.nodeValue=rendered;}
  for(const el of document.querySelectorAll('[title],[placeholder],[aria-label]')){if(el.closest('[data-i18n-ignore]'))continue;let saved=attrs.get(el);if(!saved){saved={};attrs.set(el,saved);}for(const name of ['title','placeholder','aria-label']){if(!el.hasAttribute(name))continue;const current=el.getAttribute(name),old=saved[name],source=old&&current===old.rendered?old.source:current,rendered=translate(source);saved[name]={source,rendered};if(current!==rendered)el.setAttribute(name,rendered);}}
  document.documentElement.lang=language;observer?.observe(document.body,{childList:true,subtree:true,characterData:true,attributes:true,attributeFilter:['title','placeholder','aria-label']});
 }
 observer=new MutationObserver(apply);apply();
 return {setLanguage(value){const next=value==='en'?'en':'ru';if(next!==language){language=next;apply();}},apply,t:translate,locale:()=>language==='en'?'en-US':'ru-RU'};
})();
