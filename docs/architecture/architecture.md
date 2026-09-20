# Mimari

Stellerpool üç parçadan oluşur: **Soroban kontratı** (fon ve kurallar), **web arayüzü** (cüzdan imzalı işlemler ve okuma) ve **anchor** (fiat kapısı). Zincir dışı bir sunucuya (backend) şu an ihtiyaç yoktur: durum zincirden okunur, belge içeriği zincire yazılmaz, yalnızca SHA-256 özeti yazılır.

## Bileşenler

~~~mermaid
flowchart LR
  subgraph Browser["Tarayıcı (Vue 3 + TypeScript)"]
    UI["Arayüz<br/>Home · Create · Pool"]
    SVC["services/pool.ts<br/>sözleşme istemcisi"]
    ANC["lib/anchor.ts<br/>SEP-1 · SEP-10 · SEP-24"]
    WK["Stellar Wallets Kit"]
  end
  subgraph Stellar["Stellar Testnet"]
    RPC["Soroban RPC"]
    POOL["rotating_pool<br/>Soroban kontratı (API v11)"]
    SAC["Havuz varlığı (SAC)<br/>USDC / demo varlığı"]
    HZ["Horizon"]
  end
  subgraph Anchor["Anchor (şu an SDF test anchor'ı)"]
    TOML["stellar.toml"]
    AUTH["SEP-10 web auth"]
    S24["SEP-24 interaktif"]
  end
  UI --> SVC
  UI --> ANC
  SVC -->|"simülasyon + gönderim"| RPC
  RPC --> POOL
  POOL -->|"transfer"| SAC
  SVC -->|"imza isteği"| WK
  ANC -->|"challenge imzası"| WK
  ANC --> TOML
  ANC --> AUTH
  ANC --> S24
  UI -->|"bakiye / hesap"| HZ
~~~

| Bileşen | Sorumluluk | Dosya |
|---|---|---|
| Kontrat | Havuz, tur, katkı, doğrulayıcı onayı, satıcıya ödeme, iade | `contracts/rotating_pool/src/` |
| Sözleşme istemcisi | 18 kontrat fonksiyonu (kura dahil), yeteneklerin zincirden okunması, `Result` çözme | `frontend/src/services/pool.ts` |
| Anchor istemcisi | stellar.toml keşfi, SEP-10 doğrulaması ve imzası, SEP-24 oturumu | `frontend/src/lib/anchor.ts` |
| Cüzdan | Bağlantı ve imza | `frontend/src/stores/wallet.ts` |
| Arayüz | Adım adım rehber, kura sahnesi, hikâye simülatörü, 3B sahne | `frontend/src/views/`, `components/` |

## Havuz durum makinesi

~~~mermaid
stateDiagram-v2
  [*] --> Filling: create_pool
  Filling --> Active: start_pool (herkes onayladı, grup tam)
  Filling --> Aborted: cancel_unstarted_pool (kuruluş süresi doldu)
  Active --> Completed: son tur ödendi
  Active --> Aborted: abort_pool (ek süre ya da alım süresi doldu)
  Completed --> [*]
  Aborted --> [*]
~~~

## Tur durum makinesi

~~~mermaid
stateDiagram-v2
  [*] --> Collecting
  Collecting --> Grace: mark_overdue (katkı süresi doldu)
  Collecting --> AwaitingPurchase: tüm üyeler ödedi (sabit sıra)
  Collecting --> AwaitingDraw: tüm üyeler ödedi (kura)
  Grace --> AwaitingPurchase: eksik üye cure_payment ile ödedi (sabit sıra)
  Grace --> AwaitingDraw: eksik üye cure_payment ile ödedi (kura)
  AwaitingDraw --> AwaitingPurchase: draw_recipient (herkes çağırabilir)
  AwaitingPurchase --> Settled: execute_round (doğrulayıcı eşiği + süre içinde)
  Settled --> Collecting: sonraki tur
  Grace --> [*]: abort_pool (iade)
  AwaitingDraw --> [*]: abort_pool (süre doldu, iade)
  AwaitingPurchase --> [*]: abort_pool (iade)
~~~

**Kura modu (API v10'dan beri, Testnet'te canlı):** alıcı tur başında belli değildir. Tüm üyeler kendi katkısını yatırınca tur `AwaitingDraw` olur ve alım süresi (`purchase_deadline`) burada başlar. `draw_recipient`, henüz teslim almamış üyeler arasından `env.prng()` ile bir alıcı seçer; herkes çağırabilir, kazanan zaten payını ödemiştir. Tek aday kalınca seçim deterministiktir. Rastgelelik hackathon düzeyindedir (bkz. [görev listesi](../CONTRACT_HANDOFF.md)).

## Fon akışı ve değişmezler

~~~mermaid
sequenceDiagram
  participant M as Üye
  participant C as rotating_pool
  participant V as Doğrulayıcılar
  participant S as Demo satıcısı
  M->>C: deposit (kendi tur katkısı)
  Note over C: Tur bazında ayrı muhasebe
  M->>C: propose_purchase (satıcı, tutar, belge SHA-256)
  V->>C: approve_purchase (eşik: ceil(2/3))
  M->>C: execute_round (herkes çağırabilir)
  C->>S: yalnızca bu turun tutarı
  Note over C,M: Ödeme aksarsa: mark_overdue → abort_pool → claim_refund (yalnızca mevcut tur)
~~~

- Kurucuda serbest çekim, tek taraflı sıra/satıcı değiştirme veya kod yükseltme yetkisi yoktur.
- Bir tur satıcıya ödenmeden önce `N × C` toplanmış ve tüm üyeler `paid` olmalıdır. Transfer ve durum değişimi atomiktir.
- İade hakkı yalnızca **mevcut, ödenmemiş turdaki** bizzat yatırılmış katkıdır. Önceki turlara iade hakkı yazılmaz.
- Satıcı yalnızca havuzda kayıtlı izinli demo satıcısıdır. Yeni alım önerisi eski onayları siler.

## Anchor akışı (SEP-1 / 10 / 24)

~~~mermaid
sequenceDiagram
  participant U as Kullanıcı
  participant A as Arayüz
  participant W as Cüzdan
  participant N as Anchor
  A->>N: GET /.well-known/stellar.toml (SEP-1)
  N-->>A: WEB_AUTH_ENDPOINT, TRANSFER_SERVER_SEP0024, SIGNING_KEY
  A->>N: GET /auth?account=… (SEP-10 challenge)
  N-->>A: challenge işlemi
  Note over A: Sunucu imzası, home domain, web_auth_domain doğrulanır
  A->>W: challenge'ı imzala
  W-->>A: imzalı işlem
  A->>N: POST /auth (imzalı işlem)
  N-->>A: JWT (yalnızca bellekte)
  A->>N: POST /sep24/transactions/deposit/interactive
  N-->>A: interaktif pencere adresi + işlem no
  U->>N: pencerede formu tamamlar
  A->>N: GET /sep24/transaction?id=… (durum izleme)
~~~

**Kapsam notu:** Bu akış SDF test anchor'ı ile doğrulandı (SEP-10 imzası ve SEP-24 oturumu). Test anchor'ı Türk lirası değil, test varlığı üretir. Gerçek TRY sağlayıcısı bulunduğunda yalnızca `VITE_ANCHOR_HOME_DOMAIN` değişir. Çekme yönü canlı akışta henüz yoktur.

## Tasarım kararları ve trade-off'lar

| Karar | Gerekçe | Bedeli |
|---|---|---|
| Sponsor yok | Riski gizleyen ayrı bir güvence modeli istemedik | Erken teslim alanın temerrüdü kontrat dışı bir sorun olarak kalır |
| Yalnızca mevcut tur iadesi | Satıcıya giden para kontratta değil | Geçmiş tur ödemeleri geri alınamaz |
| Zaman aşımı kendiliğinden işlem yapmaz | Soroban'da zamanlayıcı yok | Herkes ilgili fonksiyonu çağırmalı (`mark_overdue`, `abort_pool`) |
| Belge zincire yazılmaz, SHA-256 özeti yazılır | Gizlilik ve maliyet | Belgeyi doğrulayıcılar zincir dışında kontrol eder |
| Yükseltilemez kontrat | Kurucu tek taraflı kod değiştiremesin | Her değişiklik yeni kontrat ID'si demektir |
| Arayüz yetenekleri zincirden okur | Kontrat ve arayüz paralel gelişti | Uyumsuz sürümde işlem reddedilir |
| Backend yok | Durum zincirde, fon yetkisi zincir dışında olmamalı | Zincir dışı bildirim ve AI raporu için ileride servis gerekir |

## Peşinat (API v11)

`create_pool(..., down_payment, ...)` üye başına peşinat belirler (0 = kapalı, v10 ile aynı). `join_pool` bu tutarı üyeden kontrata çeker (havuz bakiyesine eklenir). Tur ödenirken satıcıya **havuz tutarı + alıcının kendi peşinatı** gider (`propose_purchase.amount` bunu ister). İptalde iade hakkı = mevcut tur katkısı (yatırdıysa) + peşinat (henüz almadıysa). Peşinat birikime sayılmaz ve teminat değildir: erken alanın sonraki taksitleri bırakma riskini kapatmaz. Arayüz, kontratın `create_pool` girdilerini zincirden okuyarak peşinat desteğini algılar; desteklemeyen eski kontratta peşinat yalnızca plan hesabıdır.
