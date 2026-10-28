//! Localized copy for provider-derived output protection.

use super::i18n::Language;
use super::provider::GuardReason;

/// Complete provider-guard copy for one supported language.
#[derive(Debug)]
pub(crate) struct GuardMessages {
    pub(crate) section_title: &'static str,
    pub(crate) label: &'static str,
    /// Active copy for known local runtimes such as Ollama and LM Studio.
    pub(crate) active_note: &'static str,
    /// Active copy for custom relay routes, including providers named OpenAI with a base URL.
    pub(crate) relay_active_note: &'static str,
    pub(crate) disabled_note: &'static str,
    pub(crate) available_note: &'static str,
    pub(crate) locked_note: &'static str,
    pub(crate) disable_confirm: &'static str,
    pub(crate) budget_follows_guarded_note: &'static str,
}

impl GuardMessages {
    #[cfg(test)]
    fn values(&self) -> [&'static str; 9] {
        [
            self.section_title,
            self.label,
            self.active_note,
            self.relay_active_note,
            self.disabled_note,
            self.available_note,
            self.locked_note,
            self.disable_confirm,
            self.budget_follows_guarded_note,
        ]
    }

    pub(crate) const fn active_note(&self, reason: Option<GuardReason>) -> &'static str {
        match reason {
            Some(GuardReason::UnverifiedRelay) => self.relay_active_note,
            Some(GuardReason::LocalCompaction) | None => self.active_note,
        }
    }
}

/// Returns provider-guard copy for a supported language.
pub(crate) const fn messages(language: Language) -> &'static GuardMessages {
    match language {
        Language::En => &EN,
        Language::ZhCn => &ZH_CN,
        Language::ZhTw => &ZH_TW,
        Language::Ja => &JA,
        Language::Ko => &KO,
        Language::Es => &ES,
        Language::Fr => &FR,
        Language::De => &DE,
        Language::PtBr => &PT_BR,
        Language::Ru => &RU,
        Language::It => &IT,
        Language::Tr => &TR,
        Language::Pl => &PL,
        Language::Nl => &NL,
        Language::Vi => &VI,
        Language::Id => &ID,
        Language::Uk => &UK,
    }
}

macro_rules! guard_messages {
    ($section:expr, $label:expr, $active:expr, $relay_active:expr, $disabled:expr, $available:expr,
     $locked:expr, $confirm:expr, $budget:expr) => {
        GuardMessages {
            section_title: $section,
            label: $label,
            active_note: $active,
            relay_active_note: $relay_active,
            disabled_note: $disabled,
            available_note: $available,
            locked_note: $locked,
            disable_confirm: $confirm,
            budget_follows_guarded_note: $budget,
        }
    };
}

const EN: GuardMessages = guard_messages!(
    "Provider compatibility",
    "Provider guard",
    "This provider uses local compaction: Codex refeeds the full history to the model. Guarded caps FastCtx's combined output per turn to preserve compaction room; it cannot verify the model catalog or context threshold.",
    "This provider is a relay whose compaction protocol and model catalog FastCtx cannot verify. Guarded caps FastCtx's combined output per turn to preserve compaction room; it cannot replace correct relay usage, compaction, or context-error handling.",
    "Protection is disabled. A local or unverified relay route can let one turn of combined tool output cross Codex's remaining compaction room.",
    "Protection is on but idle: this route is official OpenAI, Azure, or Bedrock. The selected tier remains effective; Guarded engages for local or unverified relay routes.",
    "Guarded locks the host at 10000 and the per-turn FastCtx pool at 9000. The selected tier is preserved and returns when this route no longer needs protection.",
    "Disable provider output protection?\nWithout the shared Guarded pool, combined outputs in one turn can cross Codex's compaction room. Guarded cannot make provider usage, compaction, catalog, or error handling valid. This affects Apply and new runtime sessions.",
    "auto — follows the Guarded per-turn pool's full 100% share."
);
const ZH_CN: GuardMessages = guard_messages!(
    "Provider 兼容性",
    "Provider 输出保护",
    "当前 provider 使用本地压缩：Codex 会把整段历史重新交给模型。Guarded 已限制 FastCtx 每轮注入的总输出，为压缩保留余量；它无法验证模型目录或上下文阈值。",
    "当前 provider 是 FastCtx 无法验证压缩协议与模型目录的中转。Guarded 已限制 FastCtx 每轮注入的总输出，为压缩保留余量；它不能替代中转对 usage、压缩和上下文错误码的正确实现。",
    "保护已关闭。本地或未经验证的中转路线可能让一轮工具总输出越过 Codex 剩余的压缩余量。",
    "保护已开启，当前无需收紧：这是官方 OpenAI、Azure 或 Bedrock 路线，仍使用你选择的档位；遇到本地或未经验证的中转路线时会自动启用 Guarded。",
    "Guarded 锁定为宿主 10000、FastCtx 每轮共享池 9000。你选择的档位会保留，并在路线不再需要保护时自动恢复。",
    "要关闭 provider 输出保护吗？\n没有 Guarded 共享池时，一轮内的合并输出可能越过 Codex 的压缩余量。Guarded 也不能让 provider 的 usage、压缩、目录或错误码实现自动变正确。这会影响 Apply 和新启动的运行时会话。",
    "auto：跟随 Guarded 每轮共享池的完整 100% 比例。"
);
const ZH_TW: GuardMessages = guard_messages!(
    "Provider 相容性",
    "Provider 輸出保護",
    "目前的 provider 使用本機壓縮：Codex 會把整段歷史重新交給模型。Guarded 已限制 FastCtx 每回合注入的總輸出，為壓縮保留餘量；它無法驗證模型目錄或上下文門檻。",
    "目前的 provider 是 FastCtx 無法驗證壓縮協定與模型目錄的中轉。Guarded 已限制 FastCtx 每回合注入的總輸出，為壓縮保留餘量；它不能取代中轉對 usage、壓縮與上下文錯誤碼的正確實作。",
    "保護已停用。本機或未經驗證的中轉路線，可能讓單一回合的工具總輸出越過 Codex 剩餘的壓縮餘量。",
    "保護已啟用，目前無需收緊：這是官方 OpenAI、Azure 或 Bedrock 路線，仍使用所選層級；遇到本機或未經驗證的中轉路線時會自動啟用 Guarded。",
    "Guarded 鎖定為主機 10000、FastCtx 每回合共用池 9000。所選層級會保留，並在路線不再需要保護時自動恢復。",
    "要停用 provider 輸出保護嗎？\n若沒有 Guarded 共用池，單一回合內的合併輸出可能越過 Codex 的壓縮餘量。Guarded 也不能讓 provider 的 usage、壓縮、目錄或錯誤碼實作自動正確。這會影響 Apply 與新的執行階段工作階段。",
    "auto：跟隨 Guarded 每回合共用池的完整 100% 比例。"
);
const JA: GuardMessages = guard_messages!(
    "Provider 互換性",
    "Provider 出力保護",
    "この provider はローカル圧縮を使い、Codex は履歴全体をモデルへ再送します。Guarded は圧縮余地を残すため、1 ターンの FastCtx 合計出力を制限しますが、モデルカタログやコンテキスト閾値は検証できません。",
    "この provider は、FastCtx が圧縮プロトコルとモデルカタログを検証できない中継です。Guarded は 1 ターンの FastCtx 合計出力を制限しますが、中継側の usage・圧縮・コンテキストエラー処理の正しさを代替できません。",
    "保護は無効です。ローカルまたは未検証の中継では、1 ターンのツール合計出力が Codex の残りの圧縮余地を越えることがあります。",
    "保護は有効ですが現在は待機中です。公式 OpenAI、Azure、Bedrock の経路では選択した段階が有効で、ローカルまたは未検証の中継経路で Guarded が自動的に有効になります。",
    "Guarded はホスト 10000、FastCtx のターン共有プール 9000 に固定されます。選択した段階は保持され、保護が不要な経路に戻ると自動復元されます。",
    "provider 出力保護を無効にしますか？\nGuarded 共有プールがないと、1 ターンの合計出力が Codex の圧縮余地を越えることがあります。Guarded でも provider の usage・圧縮・カタログ・エラー処理を正しくすることはできません。Apply と新しいランタイムセッションに影響します。",
    "auto：Guarded のターン共有プールの完全な 100% 割合に従います。"
);
const KO: GuardMessages = guard_messages!(
    "Provider 호환성",
    "Provider 출력 보호",
    "이 provider는 로컬 압축을 사용하므로 Codex가 전체 기록을 모델에 다시 보냅니다. Guarded는 압축 여유를 남기기 위해 한 턴의 FastCtx 총출력을 제한하지만 모델 카탈로그나 컨텍스트 임계값은 검증하지 못합니다.",
    "이 provider는 FastCtx가 압축 프로토콜과 모델 카탈로그를 검증할 수 없는 중계입니다. Guarded는 한 턴의 FastCtx 총출력을 제한하지만 중계의 usage, 압축, 컨텍스트 오류 처리가 올바른지를 대신 보장하지 못합니다.",
    "보호가 꺼져 있습니다. 로컬 또는 검증되지 않은 중계 경로에서는 한 턴의 도구 총출력이 Codex의 남은 압축 여유를 넘을 수 있습니다.",
    "보호가 켜져 있지만 현재는 대기 중입니다. 공식 OpenAI, Azure, Bedrock 경로에서는 선택한 단계가 유지되며 로컬 또는 검증되지 않은 중계 경로에서 Guarded가 자동으로 켜집니다.",
    "Guarded는 호스트 10000과 FastCtx 턴 공유 풀 9000으로 잠깁니다. 선택한 단계는 보존되며 보호가 필요 없는 경로로 바뀌면 자동 복원됩니다.",
    "provider 출력 보호를 끌까요?\nGuarded 공유 풀이 없으면 한 턴의 합산 출력이 Codex의 압축 여유를 넘을 수 있습니다. Guarded도 provider의 usage, 압축, 카탈로그, 오류 처리를 올바르게 만들 수는 없습니다. Apply와 새 런타임 세션에 영향을 줍니다.",
    "auto: Guarded 턴 공유 풀의 전체 100% 비율을 따릅니다."
);
const ES: GuardMessages = guard_messages!(
    "Compatibilidad del provider",
    "Protección del provider",
    "Este provider usa compactación local: Codex reenvía todo el historial al modelo. Guarded limita la salida total de FastCtx por turno para reservar espacio de compactación, pero no puede verificar el catálogo del modelo ni el umbral de contexto.",
    "Este provider es un relay cuyo protocolo de compactación y catálogo de modelos FastCtx no puede verificar. Guarded limita la salida total de FastCtx por turno, pero no sustituye una implementación correcta de usage, compactación y errores de contexto en el relay.",
    "La protección está desactivada. Una ruta local o de relay sin verificar puede hacer que la salida combinada de un turno supere el espacio de compactación restante de Codex.",
    "La protección está activa pero inactiva: esta ruta es OpenAI oficial, Azure o Bedrock. El nivel elegido sigue vigente; Guarded se activa para rutas locales o relays sin verificar.",
    "Guarded fija el host en 10000 y el fondo compartido por turno de FastCtx en 9000. El nivel elegido se conserva y vuelve cuando la ruta ya no necesita protección.",
    "¿Desactivar la protección de salida del provider?\nSin el fondo compartido Guarded, las salidas combinadas de un turno pueden superar el espacio de compactación de Codex. Guarded tampoco corrige usage, compactación, catálogo ni errores del provider. Afecta a Apply y a las nuevas sesiones de ejecución.",
    "auto — sigue la cuota completa del 100% del fondo Guarded por turno."
);
const FR: GuardMessages = guard_messages!(
    "Compatibilité du provider",
    "Protection du provider",
    "Ce provider utilise le compactage local : Codex renvoie tout l’historique au modèle. Guarded limite la sortie FastCtx totale par tour pour préserver une marge de compactage, sans pouvoir vérifier le catalogue du modèle ni le seuil de contexte.",
    "Ce provider est un relais dont FastCtx ne peut vérifier ni le protocole de compactage ni le catalogue des modèles. Guarded limite la sortie FastCtx totale par tour, mais ne remplace pas une gestion correcte de l’usage, du compactage et des erreurs de contexte par le relais.",
    "La protection est désactivée. Une route locale ou un relais non vérifié peut faire dépasser à la sortie combinée d’un tour la marge de compactage restante de Codex.",
    "La protection est active mais en veille : cette route est OpenAI officielle, Azure ou Bedrock. Le niveau choisi reste actif ; Guarded s’enclenche pour les routes locales ou les relais non vérifiés.",
    "Guarded verrouille l’hôte à 10000 et le pool FastCtx partagé par tour à 9000. Le niveau choisi est conservé et revient lorsque la route n’a plus besoin de protection.",
    "Désactiver la protection de sortie du provider ?\nSans le pool Guarded partagé, les sorties combinées d’un tour peuvent dépasser la marge de compactage de Codex. Guarded ne peut pas non plus corriger l’usage, le compactage, le catalogue ou les erreurs du provider. Cela touche Apply et les nouvelles sessions.",
    "auto — suit la part complète de 100 % du pool Guarded par tour."
);
const DE: GuardMessages = guard_messages!(
    "Provider-Kompatibilität",
    "Provider-Schutz",
    "Dieser Provider verwendet lokale Komprimierung: Codex reicht den gesamten Verlauf erneut an das Modell. Guarded begrenzt die gesamte FastCtx-Ausgabe pro Runde, um Komprimierungsraum zu bewahren, kann aber Modellkatalog und Kontextgrenze nicht prüfen.",
    "Dieser Provider ist ein Relay, dessen Komprimierungsprotokoll und Modellkatalog FastCtx nicht prüfen kann. Guarded begrenzt die gesamte FastCtx-Ausgabe pro Runde, ersetzt aber keine korrekte Usage-, Komprimierungs- und Kontextfehlerbehandlung des Relays.",
    "Der Schutz ist deaktiviert. Bei einer lokalen oder ungeprüften Relay-Route kann die kombinierte Tool-Ausgabe einer Runde den verbleibenden Komprimierungsraum von Codex überschreiten.",
    "Der Schutz ist aktiv, aber inaktiv: Diese Route ist offizielles OpenAI, Azure oder Bedrock. Die gewählte Stufe gilt weiter; Guarded greift bei lokalen oder ungeprüften Relay-Routen.",
    "Guarded sperrt den Host auf 10000 und den pro Runde geteilten FastCtx-Pool auf 9000. Die gewählte Stufe bleibt erhalten und kehrt zurück, sobald die Route keinen Schutz mehr benötigt.",
    "Provider-Ausgabeschutz deaktivieren?\nOhne den geteilten Guarded-Pool können kombinierte Ausgaben einer Runde den Komprimierungsraum von Codex überschreiten. Guarded kann Usage, Komprimierung, Katalog oder Fehlerbehandlung des Providers nicht korrigieren. Dies betrifft Apply und neue Laufzeitsitzungen.",
    "auto — folgt dem vollen 100-%-Anteil des Guarded-Pools pro Runde."
);
const PT_BR: GuardMessages = guard_messages!(
    "Compatibilidade do provider",
    "Proteção do provider",
    "Este provider usa compactação local: o Codex reenvia todo o histórico ao modelo. O Guarded limita a saída total do FastCtx por turno para preservar espaço de compactação, mas não verifica o catálogo do modelo nem o limite de contexto.",
    "Este provider é um relay cujo protocolo de compactação e catálogo de modelos o FastCtx não consegue verificar. O Guarded limita a saída total do FastCtx por turno, mas não substitui usage, compactação e erros de contexto corretos no relay.",
    "A proteção está desativada. Uma rota local ou um relay não verificado pode fazer a saída combinada de um turno ultrapassar o espaço de compactação restante do Codex.",
    "A proteção está ativa, mas ociosa: esta rota é OpenAI oficial, Azure ou Bedrock. O nível escolhido continua valendo; o Guarded entra em rotas locais ou relays não verificados.",
    "O Guarded fixa o host em 10000 e o pool do FastCtx compartilhado por turno em 9000. O nível escolhido é preservado e retorna quando a rota não precisa mais de proteção.",
    "Desativar a proteção de saída do provider?\nSem o pool compartilhado do Guarded, as saídas combinadas de um turno podem ultrapassar o espaço de compactação do Codex. O Guarded também não corrige usage, compactação, catálogo ou erros do provider. Isso afeta Apply e novas sessões.",
    "auto — segue a parcela completa de 100% do pool Guarded por turno."
);
const RU: GuardMessages = guard_messages!(
    "Совместимость provider",
    "Защита provider",
    "Этот provider использует локальное сжатие: Codex заново передаёт модели всю историю. Guarded ограничивает суммарный вывод FastCtx за ход, оставляя место для сжатия, но не может проверить каталог модели или порог контекста.",
    "Этот provider — посредник, протокол сжатия и каталог моделей которого FastCtx не может проверить. Guarded ограничивает суммарный вывод FastCtx за ход, но не заменяет корректную передачу usage, сжатия и ошибок контекста посредником.",
    "Защита отключена. Локальный или непроверенный маршрут через посредника может позволить суммарному выводу за ход превысить оставшееся место для сжатия Codex.",
    "Защита включена, но сейчас бездействует: это официальный маршрут OpenAI, Azure или Bedrock. Выбранный уровень действует; Guarded включается для локальных и непроверенных маршрутов.",
    "Guarded фиксирует хост на 10000, а общий пул FastCtx на ход — на 9000. Выбранный уровень сохраняется и возвращается, когда маршрут больше не требует защиты.",
    "Отключить защиту вывода provider?\nБез общего пула Guarded суммарный вывод за ход может превысить место для сжатия Codex. Guarded также не исправляет usage, сжатие, каталог или ошибки provider. Это влияет на Apply и новые сеансы.",
    "auto — следует полной доле 100% пула Guarded на ход."
);
const IT: GuardMessages = guard_messages!(
    "Compatibilità provider",
    "Protezione provider",
    "Questo provider usa la compattazione locale: Codex rimanda l’intera cronologia al modello. Guarded limita l’output totale di FastCtx per turno per conservare spazio di compattazione, ma non verifica il catalogo del modello o la soglia di contesto.",
    "Questo provider è un relay di cui FastCtx non può verificare protocollo di compattazione e catalogo modelli. Guarded limita l’output totale di FastCtx per turno, ma non sostituisce usage, compattazione e gestione degli errori di contesto corretti nel relay.",
    "La protezione è disattivata. Una rotta locale o un relay non verificato può far superare all’output combinato di un turno lo spazio di compattazione residuo di Codex.",
    "La protezione è attiva ma a riposo: questa rotta è OpenAI ufficiale, Azure o Bedrock. Resta valido il livello scelto; Guarded scatta per rotte locali o relay non verificati.",
    "Guarded blocca l’host a 10000 e il pool FastCtx condiviso per turno a 9000. Il livello scelto viene conservato e torna quando la rotta non richiede più protezione.",
    "Disattivare la protezione output del provider?\nSenza il pool Guarded condiviso, gli output combinati di un turno possono superare lo spazio di compattazione di Codex. Guarded non corregge usage, compattazione, catalogo o errori del provider. Riguarda Apply e le nuove sessioni.",
    "auto — segue la quota completa del 100% del pool Guarded per turno."
);
const TR: GuardMessages = guard_messages!(
    "Provider uyumluluğu",
    "Provider koruması",
    "Bu provider yerel sıkıştırma kullanır; Codex tüm geçmişi modele yeniden gönderir. Guarded, sıkıştırma alanı bırakmak için tur başına toplam FastCtx çıktısını sınırlar; model kataloğunu veya bağlam eşiğini doğrulayamaz.",
    "Bu provider, sıkıştırma protokolü ve model kataloğu FastCtx tarafından doğrulanamayan bir aktarıcıdır. Guarded tur başına toplam FastCtx çıktısını sınırlar; aktarıcının usage, sıkıştırma ve bağlam hatası işlemesini doğru yapmasının yerini tutmaz.",
    "Koruma kapalı. Yerel veya doğrulanmamış bir aktarıcı rotasında tek turun birleşik araç çıktısı Codex’in kalan sıkıştırma alanını aşabilir.",
    "Koruma açık ama şu an devrede değil: bu rota resmi OpenAI, Azure veya Bedrock’tır. Seçilen kademe geçerli kalır; Guarded yerel veya doğrulanmamış aktarıcı rotalarında devreye girer.",
    "Guarded host’u 10000, tur başına paylaşılan FastCtx havuzunu 9000 olarak kilitler. Seçilen kademe korunur ve rota artık koruma gerektirmediğinde geri döner.",
    "Provider çıktı koruması kapatılsın mı?\nPaylaşılan Guarded havuzu olmadan bir turun birleşik çıktıları Codex’in sıkıştırma alanını aşabilir. Guarded provider’ın usage, sıkıştırma, katalog veya hata işlemesini düzeltemez. Apply ve yeni çalışma zamanı oturumlarını etkiler.",
    "auto — tur başına Guarded havuzunun tam %100 payını izler."
);
const PL: GuardMessages = guard_messages!(
    "Zgodność providera",
    "Ochrona providera",
    "Ten provider używa lokalnej kompakcji: Codex ponownie przekazuje modelowi całą historię. Guarded ogranicza łączny wynik FastCtx na turę, aby zachować miejsce na kompakcję, lecz nie weryfikuje katalogu modelu ani progu kontekstu.",
    "Ten provider jest pośrednikiem, którego protokołu kompakcji i katalogu modeli FastCtx nie może zweryfikować. Guarded ogranicza łączny wynik FastCtx na turę, ale nie zastępuje poprawnego usage, kompakcji i obsługi błędów kontekstu przez pośrednika.",
    "Ochrona jest wyłączona. Lokalna lub niezweryfikowana trasa pośrednika może sprawić, że łączny wynik narzędzi w jednej turze przekroczy pozostałe miejsce na kompakcję Codex.",
    "Ochrona jest włączona, ale bezczynna: to oficjalna trasa OpenAI, Azure lub Bedrock. Wybrany poziom obowiązuje; Guarded włącza się dla tras lokalnych i niezweryfikowanych pośredników.",
    "Guarded blokuje host na 10000, a współdzieloną pulę FastCtx na turę na 9000. Wybrany poziom zostaje zachowany i wraca, gdy trasa nie wymaga już ochrony.",
    "Wyłączyć ochronę wyjścia providera?\nBez wspólnej puli Guarded łączne wyniki jednej tury mogą przekroczyć miejsce na kompakcję Codex. Guarded nie naprawia usage, kompakcji, katalogu ani błędów providera. Dotyczy Apply i nowych sesji.",
    "auto — podąża za pełnym udziałem 100% puli Guarded na turę."
);
const NL: GuardMessages = guard_messages!(
    "Providercompatibiliteit",
    "Providerbescherming",
    "Deze provider gebruikt lokale compactie: Codex voert de hele geschiedenis opnieuw aan het model. Guarded begrenst de totale FastCtx-uitvoer per beurt om compactieruimte te bewaren, maar kan de modelcatalogus of contextdrempel niet verifiëren.",
    "Deze provider is een relay waarvan FastCtx het compactieprotocol en de modelcatalogus niet kan verifiëren. Guarded begrenst de totale FastCtx-uitvoer per beurt, maar vervangt geen correcte usage-, compactie- en contextfoutafhandeling door de relay.",
    "Bescherming is uitgeschakeld. Bij een lokale of niet-geverifieerde relayroute kan de gecombineerde tooluitvoer van één beurt de resterende compactieruimte van Codex overschrijden.",
    "Bescherming is ingeschakeld maar rust: dit is een officiële OpenAI-, Azure- of Bedrock-route. Het gekozen niveau blijft gelden; Guarded treedt in werking bij lokale of niet-geverifieerde relayroutes.",
    "Guarded vergrendelt de host op 10000 en de per beurt gedeelde FastCtx-pool op 9000. Het gekozen niveau blijft bewaard en keert terug zodra de route geen bescherming meer nodig heeft.",
    "Provider-uitvoerbescherming uitschakelen?\nZonder de gedeelde Guarded-pool kan gecombineerde uitvoer van één beurt de compactieruimte van Codex overschrijden. Guarded corrigeert usage, compactie, catalogus of fouten van de provider niet. Dit raakt Apply en nieuwe runtimesessies.",
    "auto — volgt het volledige aandeel van 100% van de Guarded-pool per beurt."
);
const VI: GuardMessages = guard_messages!(
    "Tương thích provider",
    "Bảo vệ provider",
    "Provider này dùng nén cục bộ: Codex gửi lại toàn bộ lịch sử cho mô hình. Guarded giới hạn tổng đầu ra FastCtx mỗi lượt để chừa khoảng nén, nhưng không thể xác minh danh mục mô hình hay ngưỡng ngữ cảnh.",
    "Provider này là relay có giao thức nén và danh mục mô hình mà FastCtx không thể xác minh. Guarded giới hạn tổng đầu ra FastCtx mỗi lượt, nhưng không thay thế việc relay triển khai đúng usage, nén và lỗi ngữ cảnh.",
    "Bảo vệ đã tắt. Tuyến cục bộ hoặc relay chưa xác minh có thể khiến tổng đầu ra công cụ trong một lượt vượt khoảng nén còn lại của Codex.",
    "Bảo vệ đang bật nhưng chưa cần dùng: đây là tuyến OpenAI chính thức, Azure hoặc Bedrock. Mức đã chọn vẫn có hiệu lực; Guarded bật cho tuyến cục bộ hoặc relay chưa xác minh.",
    "Guarded khóa host ở 10000 và nhóm FastCtx dùng chung mỗi lượt ở 9000. Mức đã chọn được giữ lại và trở về khi tuyến không còn cần bảo vệ.",
    "Tắt bảo vệ đầu ra provider?\nKhông có nhóm Guarded dùng chung, đầu ra kết hợp trong một lượt có thể vượt khoảng nén của Codex. Guarded cũng không sửa được usage, nén, danh mục hay lỗi của provider. Ảnh hưởng đến Apply và các phiên chạy mới.",
    "auto — theo toàn bộ tỷ lệ 100% của nhóm Guarded mỗi lượt."
);
const ID: GuardMessages = guard_messages!(
    "Kompatibilitas provider",
    "Perlindungan provider",
    "Provider ini memakai kompaksi lokal: Codex mengirim ulang seluruh riwayat ke model. Guarded membatasi total keluaran FastCtx per giliran untuk menyisakan ruang kompaksi, tetapi tidak dapat memverifikasi katalog model atau ambang konteks.",
    "Provider ini adalah relay yang protokol kompaksi dan katalog modelnya tidak dapat diverifikasi FastCtx. Guarded membatasi total keluaran FastCtx per giliran, tetapi tidak menggantikan penerapan usage, kompaksi, dan galat konteks yang benar di relay.",
    "Perlindungan dinonaktifkan. Rute lokal atau relay yang belum diverifikasi dapat membuat keluaran alat gabungan satu giliran melewati sisa ruang kompaksi Codex.",
    "Perlindungan aktif tetapi belum diperlukan: ini rute OpenAI resmi, Azure, atau Bedrock. Tingkat pilihan tetap berlaku; Guarded aktif untuk rute lokal atau relay yang belum diverifikasi.",
    "Guarded mengunci host pada 10000 dan pool FastCtx bersama per giliran pada 9000. Tingkat pilihan disimpan dan kembali saat rute tidak lagi memerlukan perlindungan.",
    "Nonaktifkan perlindungan keluaran provider?\nTanpa pool Guarded bersama, keluaran gabungan satu giliran dapat melewati ruang kompaksi Codex. Guarded juga tidak memperbaiki usage, kompaksi, katalog, atau galat provider. Ini memengaruhi Apply dan sesi runtime baru.",
    "auto — mengikuti porsi penuh 100% pool Guarded per giliran."
);
const UK: GuardMessages = guard_messages!(
    "Сумісність provider",
    "Захист provider",
    "Цей provider використовує локальне стиснення: Codex знову передає моделі всю історію. Guarded обмежує сумарний вивід FastCtx за хід, залишаючи місце для стиснення, але не може перевірити каталог моделі чи поріг контексту.",
    "Цей provider є посередником, протокол стиснення й каталог моделей якого FastCtx не може перевірити. Guarded обмежує сумарний вивід FastCtx за хід, але не замінює правильну передачу usage, стиснення та помилок контексту посередником.",
    "Захист вимкнено. Локальний або неперевірений маршрут через посередника може дозволити сумарному виводу за хід перевищити залишок для стиснення Codex.",
    "Захист увімкнено, але наразі бездіяльний: це офіційний маршрут OpenAI, Azure або Bedrock. Вибраний рівень діє; Guarded вмикається для локальних і неперевірених маршрутів.",
    "Guarded фіксує хост на 10000, а спільний пул FastCtx на хід — на 9000. Вибраний рівень зберігається й повертається, коли маршрут більше не потребує захисту.",
    "Вимкнути захист виводу provider?\nБез спільного пулу Guarded сумарний вивід за хід може перевищити простір стиснення Codex. Guarded також не виправляє usage, стиснення, каталог або помилки provider. Це впливає на Apply і нові сеанси.",
    "auto — дотримується повної частки 100% пулу Guarded на хід."
);

#[cfg(test)]
mod tests {
    use super::messages;
    use crate::control::i18n::ALL_LANGUAGES;

    #[test]
    fn every_language_has_complete_provider_guard_copy() {
        for language in ALL_LANGUAGES {
            let messages = messages(language);
            assert!(
                messages
                    .values()
                    .iter()
                    .all(|value| !value.trim().is_empty()),
                "{} has an empty provider-guard translation",
                language.code()
            );
            // The section header sits directly above the item row, so identical text
            // renders as a duplicated line rather than a heading and its setting.
            assert_ne!(
                messages.section_title,
                messages.label,
                "{} repeats the provider-guard label as its section header",
                language.code()
            );
            assert_ne!(
                messages.active_note,
                messages.relay_active_note,
                "{} does not distinguish local compaction from an unverified relay",
                language.code()
            );
            assert!(messages.locked_note.contains("10000"));
            assert!(messages.locked_note.contains("9000"));
            assert!(messages.disable_confirm.contains("Guarded"));
            assert!(messages.budget_follows_guarded_note.contains("100"));
            assert!(messages.budget_follows_guarded_note.contains('%'));
        }
    }
}
