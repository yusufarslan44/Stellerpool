# Stellerpool — şeffaf grup tasarruf havuzu (Stellar / Soroban)

> Birlikte biriktir. Her şeyi doğrula. — *Save together. Verify everything.*

Rise In x Stellar **Pro Hackathon 2026** projesi · Track: **Genesis** (teslimde seçilecek) · Ağ: **Stellar Testnet**

## Neden? (Narrative "Why")

Türkiye'de ev, araç ya da ortak bir hedef için grup halinde birikim yapmak yaygındır: altın günleri, tasarruf finansman şirketlerinin (Eminevim, Fuzul gibi) çekilişli ya da sıralı grupları. Ortak sorun **güvendir**: para bir kişinin ya da bir kurumun elindedir, kurallar herkese görünmez, biri ödemeyi bırakınca ne olacağı belirsizdir.

Stellerpool bu koordinasyonu **Soroban akıllı sözleşmesine** taşır. Katkılar tur bazında kontratta kilitlenir, kurucunun ortak parayı çekme yetkisi yoktur, ödeme yalnızca üyelerin onayladığı kurallar ve doğrulayıcı onayı sonrası, önceden belirlenmiş satıcıya gider. Ödeme aksarsa tur durur ve yalnızca o turun katkıları sahiplerine iade edilir. **Hedef kullanıcı:** birbirini tanıyan, ortak bir alım için sırayla ya da kurayla birikim yapmak isteyen küçük gruplar.

## Bugün ne çalışıyor, ne çalışmıyor?

| Alan | Durum |
|---|---|
| Soroban kontratı (sponsorsuz, **API v11**: kura + 30 üye + **peşinat**) | ✅ Testnet'te yayında, 33 birim testi, canlı senaryolar koşuldu (kura + peşinat tam akış, iptal + iade, kurulamayan havuzun peşinat iadesi). Önceki v10 kanıtları aşağıda
| Arayüz ↔ kontrat | ✅ Canlı kontrata karşı doğrulandı: üç havuz (sıralı, iptal, kura) okunup çiziliyor; `create_pool` (kura 24/30 üye, sıralı 4 üye) zincire gönderilmeden simüle edildi, 31 üye reddedildi. ⚠️ Cüzdan imzalı yazma akışı (havuz kur, öde, onayla, kura çek) bir cüzdanla uçtan uca henüz denenmedi |
| Anchor | ✅ **Kendi işlettiğimiz SEP-1 / SEP-10 / SEP-24 anchor'ı** (`https://anchor-stellerpool.arslanyusuf.com`, Testnet, Node.js/TypeScript, kaynak: [`backend/`](backend/)). `TRYT` test varlığını basar (issuer `GBM3V2AHDEF3APOIKRCXCKVNW3UA2VWMSLTGELWRLBLIFSV7OVME6HOC`, SAC `CDATFFDUVSMP2OC6JKIRWJDUHFTNMWJ2ZXLUCDUTRD7FYXANOV7RXB3H`). Havuz sayfasında **katkı adımının içinde**: bakiyesi yetmeyen üye anchor ile havuzun kendi varlığını yükler. Uçtan uca doğrulandı (SEP-10 girişi, SEP-24 yatırma, gerçek on-chain TRYT transferi). ❌ **Türk lirası değil**: `TRYT` TRY'yi yalnızca temsil eden bir test varlığıdır, gerçek banka rayı yoktur. Cüzdan imzalı yatırma bir cüzdanla denenmedi |
| Kura modu ve 30 üye | ✅ Kontratta canlı ([görev listesi](docs/CONTRACT_HANDOFF.md)); arayüzde "Kura" seçilebilir. Rastgelelik hackathon düzeyindedir |
| Peşinat | ✅ Kontratta: üye başına, katılırken yatırılır, sıran gelince alımına eklenip satıcıya gider, iptalde harcanmamışsa iade edilir. Şirketlerdeki gibi birikime sayılmaz ve teminat değildir |
| Mainnet | Salt okunur tanıtım derlemesi; cüzdan imzası veya fon işlemi yok |
| Gerçek TL giriş/çıkışı | ❌ **Hackathon'un çekirdek gereksinimi karşılanmıyor**, doğrulanmış bir TRY anchor'ı bulunamadı ([ayrıntı](docs/altin-gunu-legal-boundary.md)) |
| AI denetçisi | ❌ Çalışan entegrasyon yok, yol haritasında |

## Kontrat ve dağıtım kanıtı (Testnet)

### Güncel kontrat: API v11 (kura + 30 üye + peşinat)

- **Kontrat ID:** [`CD23I5ZNVHOUBJOHODHHEW6NH7NB3FDBQ2DE2C2TST5BZM4KTJLBPK33`](https://stellar.expert/explorer/testnet/contract/CD23I5ZNVHOUBJOHODHHEW6NH7NB3FDBQ2DE2C2TST5BZM4KTJLBPK33) · API sürümü 11 · WASM SHA-256 `0f7ae90a…3b14` · 34.982 bayt · [yayın işlemi `7cc3648b…4dec`](https://stellar.expert/explorer/testnet/tx/7cc3648b7c6a6c163911a03eaa9406ecd075f8e9dcb9e4a63ac3af2417154dec)
- **Değişiklik (v10 → v11):** `create_pool`'a `down_payment` (üye başına, 0 = kapalı) eklendi. `join_pool` peşinatı kontrata çeker; `propose_purchase` / `execute_round` alım tutarını *havuz + alıcının kendi peşinatı* olarak ister ve satıcıya bunu öder; `claim_refund` / `get_member_status` iade hakkına henüz harcanmamış peşinatı ekler; kurulamayan havuzda (`cancel_unstarted_pool`) katılanlar peşinatlarını geri alır. Yeni hata `InvalidDownPayment`. Peşinat 0 ise davranış v10 ile aynıdır.
- **Birim testleri:** 33 (27 mevcut + 6 yeni): peşinatın katılırken çekilmesi ve alıma eklenmesi, kura modunda toz kalmaması, iptalde "mevcut tur katkısı + harcanmamış peşinat" iadesi, kurulamayan havuz iadesi, yetersiz bakiyeyle katılamama, negatif peşinat reddi.
- **Canlı doğrulama (20 Eylül 2026, `TRYT` varlığı, arayüzün servis katmanı ve oluşturma formu, tek kullanımlık test anahtarlarıyla imzalı; gerçek Freighter ile deneme yapılmadı):**
  - [Havuz #1](https://stellerpool.arslanyusuf.com/pool/1): 3 üyeli **kura + 6 TRYT peşinat**, oluşturma formundan kuruldu ve tamamlandı. Katılırken 3 × 6 TRYT kontrata çekildi; her turda yalnızca havuz tutarıyla (30) yapılan alım önerisi **reddedildi**, kazananın peşinatı eklenmiş tutarla (36) kabul edildi; satıcıya turda 36, toplam 108 gitti; kazananlar M2 → M3 → M1 (tekrar yok); sonunda kontrat bakiyesi **0**.
  - [Havuz #3](https://stellerpool.arslanyusuf.com/pool/3): **iptal + iade**. 3 üye, peşinat 5, katkı 10. Tur 1'de M1 aldı, 2. turda M3 ödemedi → ek süre → iptal. İade hakları M1 = 10 (aldı, yalnızca bu tur), M2 = 15 (katkı + peşinat), M3 = 5 (yalnızca peşinat); üçü de tam aldı, ikinci iade reddedildi.
  - [Havuz #4](https://stellerpool.arslanyusuf.com/pool/4): **kurulamayan havuz**. Kuruluş süresi doldu, iptal edildi, katılan iki üye 4 TRYT peşinatını tam geri aldı.
  - Havuz #2, test sürelerinin kısa ayarlanması yüzünden yarım kalmış bir denemedir (iptal edilip iade ettirildi); kanıt sayılmaz.

### Önceki sürüm: API v10 (kanıt olarak zincirde duruyor)

- **Kontrat ID:** [`CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB`](https://stellar.expert/explorer/testnet/contract/CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB) · API sürümü 10 · WASM SHA-256 `6cc5e1a0…12b4` · 34.070 bayt
- **Yükleme:** [`d0f9e5cb…7867`](https://stellar.expert/explorer/testnet/tx/d0f9e5cbbf023a9dde46ab2017b362f9ea8dc6a5ac643b109cabd304ba647867) · **Örnek oluşturma:** [`5c7ae114…91a1`](https://stellar.expert/explorer/testnet/tx/5c7ae11430ec06055f612027ab046888f88ddcd83d3addd6c6328bd92d7a91a1)
- **Havuz #1 (sıralı, mutlu yol):** [`create_pool`](https://stellar.expert/explorer/testnet/tx/7f14ac89fcc63b43396c92a1770258fd9b4e9e1a6db5499c26e4c0d1fb3ee94e) → 1. tur [`RoundPaid`](https://stellar.expert/explorer/testnet/tx/78bc813cae56605d08dedf5cf303e158f5a0731517663e1ac2177ae2ef177d2b) → 2. tur [`RoundPaid + PoolCompleted`](https://stellar.expert/explorer/testnet/tx/ad5a8e6e9a379816c18fb23f84d6a49f497b42a54987f2c861c9968bb3993a01)
- **Havuz #2 (kanonik risk senaryosu):** [`create_pool`](https://stellar.expert/explorer/testnet/tx/5c20843227aea3656ca45c6a1644b6bf95df1fec033e3e4d67514d13e23755cd) → 1. tur ödendi [`RoundPaid`](https://stellar.expert/explorer/testnet/tx/a00b3cf770d21634f64bebbbf75b46a897840061c20d6a2d3be33db06eb70ad6) → 2. turda yalnızca bir üye yatırdı [`deposit`](https://stellar.expert/explorer/testnet/tx/f513244596c4ba07d929fa8820d9aa92e1965b2ae5efdfe589b985528fa048fc) → süre aşımı, `mark_overdue`, `abort_pool` → yalnızca o üye iadesini çekti [`claim_refund`](https://stellar.expert/explorer/testnet/tx/d4171a4866f9e7fa15db371c00c26737ed132c33baba0b3363f482bcf4d60fd1); ilk turda alan üyenin payı geri alınamadı
- **Havuz #3 (kura):** [`create_pool`](https://stellar.expert/explorer/testnet/tx/38dec31c1c9cfc2388ce1f6adda32ad7d97c1ecd77e245b9f6226a52d195e554) (`order_mode: Draw`) → iki üye yatırdı, tur kura bekliyor [`RoundAwaitingDraw`](https://stellar.expert/explorer/testnet/tx/b1b05c4dfc1c3d40eb5b12572b41fa640c1385e491b592a774a872798756f4d4) → **havuz üyesi olmayan bir hesap** kurayı çekti [`draw_recipient`](https://stellar.expert/explorer/testnet/tx/11845a01bbfe010755af7e74588b408b469066ee8f9247dd668d12fdd0ea2822) → [`execute_round`](https://stellar.expert/explorer/testnet/tx/a253b14aee6bbd21329eb085ac7532fb984cabe3a92ee44f0bda698961e82609) → 2. turda kalan tek aday deterministik seçildi [`draw_recipient`](https://stellar.expert/explorer/testnet/tx/a9266d236b33ab5b262942e5c9734f81e6dbcb641eb040d199866f7193ed74f9) → [`execute_round`](https://stellar.expert/explorer/testnet/tx/22b4c62efd755a75ca4c4d1f8a467c9a02f48890c49dba1f3796b50054bb72fa)
- **30 üye kaynak ölçümü** (yerel test, gerçek Wasm ana bilgisayarı için alt sınır): `deposit` ≈ 1,1 M, `draw_recipient` ≈ 1,6 M, `execute_round` ≈ 1,4 M komut (Mainnet sınırı 400 M); okuma/yazma girdileri tek haneli. Gerçek 30 kişilik Testnet koşusu yapılmadı.
- Canlı havuzlar #1–#3 kontrat ekibinin bastığı `STLP` demo varlığıyla kuruldu (SAC `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`). Canlı sitede yeni havuzlar anchor'ın `TRYT` varlığıyla kurulur; arayüz her havuzun kendi token'ını okur, etiket ve bakiye buna göre gösterilir.
- Önceki üç kontrat örneği (iki sponsorlu, bir v9 sponsorsuz) zincirde durur; **teslimde geçerli olan güncel kontrat yukarıdaki v11'dir; v10 kanıt olarak durur.** Ayrıntı ve karar gerekçeleri: [IMPLEMENTATION_LOG.md](docs/IMPLEMENTATION_LOG.md) "Phase 12" ve "Phase 13".
- **Demo URL:** https://stellerpool.arslanyusuf.com (Testnet, v11 kontratına bağlı; kura + peşinat kanıtı için `/pool/1`). Yalnızca Testnet, gerçek para yok.
- **Uçtan uca arayüz doğrulaması (v10 kontratı, 20 Eylül 2026, `TRYT` ile; bu havuzlar v10 örneğindedir, canlı site artık v11'i gösterir):** Arayüzün kendi butonları ve servis katmanı, cüzdan uzantısı yerine **tek kullanımlık test anahtarlarıyla imzalatılarak** koşturuldu (gerçek Freighter ile deneme henüz yapılmadı):
  - Havuz #5 (v10): 3 üyeli **kura** havuzu, oluşturma formundan kuruldu. Katıl → şartlar → onay → başlat → katkı → **kura** → alım önerisi → 2 doğrulayıcı onayı → tutar satıcıya. Kazananlar sırayla M1, M3, M2: her tur yalnızca henüz almamışlar arasından çıktı, tekrar yok. Üye M3'ün bakiyesi arayüzdeki anchor akışıyla (SEP-10 girişi + SEP-24 yatırma) 0'dan 40 TRYT'ye yüklendi. Sonda satıcı 90 TRYT aldı, kontrat bakiyesi 0.
  - Havuz #6 (v10): 2 üyeli **sabit sıra** havuzu, onaylanan sıra birebir uygulandı, iki tur ödendi.
  - Havuz #7 (v10): **iptal + iade**. Ödemeyen üye yüzünden katkı süresi doldu → ek süre → iptal; yalnızca ödeyen üye katkısını geri aldı, ödemeyen üyenin ve ikinci iade denemesi reddedildi, kontrat bakiyesi 0.

## Havuz kuralı

- Davetli, sabit üyeli grupta katkı, takvim ve sıra üyelerin aynı sürümü onaylamasıyla kilitlenir. Kurucu tek başına değiştiremez, fon çekemez.
- Her turda herkes **kendi** katkısını yatırmadan o turun tahsisatı açılmaz. Ödeme gecikirse ek süre verilir; karşılanmazsa tur durdurulur.
- O tur henüz satıcıya ödenmediyse yalnızca **o turda yatırılan katkılar** iade edilir. Satıcıya gitmiş katkılar kontrattan geri alınamaz.
- Demo tahsisatı yalnızca izinli test satıcısına, alım önerisi ve doğrulayıcı eşiği (2/3) onayı sonrasında gider. Alım için ayrı son tarih vardır.
- Ayrı sponsor, avans veya platform garantisi yoktur.
- **Alıcı iki yolla belirlenir:** üyelerin onayladığı sabit sıra ya da **kura** (her tur, henüz teslim almamış üyeler arasından, tüm katkılar tamamlanınca; herkes çekebilir, kazanan zaten payını ödemiştir). Grup **2–30 üye**. Kura rastgeleliği hackathon düzeyindedir (`env.prng()`).

**Açık ekonomik risk:** Dört üye 10'ar birim yatırıp ilk turda 40 birim A'nın satıcısına ödenirse havuzda ilk turun parası kalmaz. A sonraki turda ödemezse yeni tur durur; B, C ve D'nin ilk tur katkıları kontrattan geri alınamaz. Üye sayısını büyütmek bunu çözmez. Bu risk arayüzde ve hikâye bölümünde görünür bırakıldı. [Ayrıntı](docs/plan.md).

## Fuzul Ev / Oto ve Eminevim ile karşılaştırma

Tasarruf finansman şirketleri faizsiz çalışır: müşteri sözleşme tutarını vadeye bölerek taksit öder, teslim sırası çekilişle ya da ödeme oranına göre belirlenir, şirket tahsisatı satıcıya öder. Maliyet faiz değil tek seferlik **organizasyon ücretidir**. Aşağıdaki şirket sütunu kamuya açık sayfalardan özetlenmiş **tipik** değerlerdir ([Fuzul Ev SSS](https://www.fuzulev.com.tr/merak-edilenler), [konut finansmanı](https://www.fuzulev.com.tr/ev-finansmani), [hesaplama rehberi](https://www.tasarruffinansmani.com/blog/fuzulev-hesaplama-rehberi)); oranlar kaynaklara ve sözleşmeye göre değişir, bu bir tavsiye değildir.

| | Tasarruf finansman şirketleri (tipik) | Stellerpool (Testnet prototipi) |
|---|---|---|
| Fon | Şirketin ayrılmış fon havuzu | Soroban kontratında tur bazında kilitli, kurucu çekemez |
| Taksit | (Sözleşme tutarı + organizasyon ücreti) ÷ vade | Hedef tutar ÷ kişi sayısı, ek ücret yok |
| **Peşinat** | İsteğe bağlı; şirkete yatar, teslimi öne çeker ya da vadeyi kısaltır | **Kontratta, üye başına.** Katılırken yatırılır, sıran gelince alımına eklenip satıcıya gider, iptalde harcanmamışsa iade edilir. Yükseldikçe havuz hedefi küçülür, vade ve kişi sayısı azalır. Şirketlerdeki gibi birikime sayılmaz, teminat değildir |
| **Ücret** | Tek seferlik organizasyon ücreti (yaklaşık %7–14), cayınca iade edilmez | **Yok**, kimseye pay ayrılmaz |
| Vade ve grup büyüklüğü | Genelde 40–240 ay; grubu şirket belirler | Kişi sayısı kadar tur (2–30). **Otomatik:** havuz hedefi ÷ ödeyebileceğin taksit; istenirse elle belirlenir |
| **Kura zamanı** | Çekilişli modelde her ay noter kontrolünde, ilk aydan itibaren; çekilişsizde teslim tarihi sözleşmede sabit | Her turda herkes taksidini yatırınca, herkesin çağırabileceği zincir işlemiyle; tarih sabit değil, noter yok |
| Alıcı | Sıra ya da çekiliş | Onaylanan sabit sıra ya da kura (hackathon düzeyi rastgelelik) |
| Kurallar | Sözleşme, şirkete bağlı | Herkese görünür, zincirde doğrulanabilir |
| Teslim | Şirket tahsisatı hak edene öder | Doğrulayıcı onayıyla yalnızca izinli demo satıcısına |
| Ödeme aksarsa | 6 aya kadar taksit dondurma, teslim erteleme | Ek süre → tur durur → mevcut tur iadesi |
| Teslim sonrası güvence | İpotek/rehin, şirket taahhüdü | **Yok** (açık risk) |
| Lisans | BDDK lisanslı (6361 sayılı Kanun) | **Yok**, Testnet, gerçek para yok |

Arayüzde bu fark oluşturma formunda ("Fuzul Ev / Oto gibi şirketlerden farkı ne?"), tur takviminde ("Bir tur nasıl geçer?") ve ana sayfa SSS'sinde anlatılır. **Organizasyon ücreti yoktur. Peşinat v11 kontratındadır ve yalnızca kendi alımına gider;** erken teslim alanın sonraki taksitleri bırakma riskini kapatmaz (teminat değildir). Yeni bir kontrat yayınında eski örnekler eski kontratta kalır, bu yüzden arayüz kontratın gerçek yeteneğini (`create_pool` girdileri) zincirden okur.

Bu bir lisanslı finansman ürününün yerine geçmez. Ayrıntı ve hukuki sınırlar: [altin-gunu-legal-boundary.md](docs/altin-gunu-legal-boundary.md), [legal-ai-path.md](docs/legal-ai-path.md).

## Akış ve mimari

Mimari diyagram, bileşenler ve durum makinesi: [docs/architecture/architecture.md](docs/architecture/architecture.md).

~~~mermaid
flowchart TD
  A[Havuz kurulur] --> B[Üyeler koşulları onaylar]
  B --> C[Her üye bu turun katkısını yatırır]
  C --> D{Tüm katkılar geldi mi?}
  D -->|Hayır, ek süre doldu| E[Bu turun katkıları iade edilir]
  D -->|Evet| F[Satıcı ve belge önerilir]
  F --> G{Doğrulayıcı onayı ve süre uygun mu?}
  G -->|Hayır| E
  G -->|Evet| H[Bu turun tutarı demo satıcısına ödenir]
  H --> C
~~~

## Teknoloji ve çalıştırma

| Klasör | Amaç |
|---|---|
| `contracts/` | Rust/Soroban `rotating_pool` kontratı ve testleri |
| `frontend/` | Vue 3, TypeScript, Vite, Tailwind, Stellar Wallets Kit, three.js |
| `backend/` | Gerekirse zincir dışı doğrulama servisi (şu an boş) |
| `scripts/` | Testnet dağıtım ve demo betikleri |
| `docs/` | Ürün, hukuk, mimari ve devir belgeleri |

**Arayüz:** `cd frontend && npm ci`, `.env` içine `VITE_ROTATING_POOL_CONTRACT_ID=<yukarıdaki ID>` yazın, `npm run dev`. Mainnet'e bağlanan işlemsiz tanıtım derlemesi için `VITE_STELLAR_NETWORK=mainnet npm run build`. İsteğe bağlı: `VITE_ANCHOR_HOME_DOMAIN` (varsayılan SDF test anchor'ı), `VITE_MAX_MEMBERS` (varsayılan 30; eski v9 kontratına bağlanıyorsanız 12).
**Kontrat:** `cargo test --workspace`; Testnet dağıtımı için [scripts/README.md](scripts/README.md).

## Stellar entegrasyonları

- **Soroban akıllı kontrat** (Rust): havuz, tur ve iade muhasebesi, doğrulayıcı eşiği.
- **Stellar Wallets Kit**: cüzdan bağlama ve imza.
- **Anchor (SEP-1/10/24)**: kendi servisimiz (`backend/`, `anchor-stellerpool.arslanyusuf.com`). stellar.toml keşfi, cüzdan imzalı SEP-10 girişi, SEP-24 interaktif yatırma; ana sayfada tam anlatım, havuz sayfasında katkıdan önce "bakiye yükle" adımı olarak akışın içinde. Havuz varlığı yalnızca anchor aynı kod ve ihraççıyı sunuyorsa önerilir. Arayüz `TRYT`'yi gerçek TL olarak göstermez ("TRY temsili test varlığı · gerçek TL değil"). Bu bir demo anchor'ıdır: "TRY yatırdım" onayı bankayı değil sunucunun kendisini tetikler; gerçek bir lisanslı TRY anchor'ı bağlanınca yalnızca `VITE_ANCHOR_HOME_DOMAIN` değişir.
- **Stellar Asset Contract (SAC)**: havuz varlığı (canlı sitede `TRYT`; eski canlı havuzlarda `STLP`; yapılandırmayla Testnet USDC).
- **Stellar SDK 17 / Soroban RPC / Horizon**: sözleşme istemcisi ve hesap okumaları.

## Tasarım kararları ve çözülen zorluklar

- **Sponsor kaldırıldı:** İlk tasarım sponsor güvencesi içeriyordu; "para zaten kilitli" sezgisiyle çeliştiği için ve gerçek riski gizlediği için kaldırıldı. Yerine riski açıkça gösteren sponsorsuz model geldi.
- **Yalnızca mevcut tur iade edilir:** Satıcıya ödenmiş turun parası kontratta değildir. Bu kısıt saklanmak yerine arayüzde kendiliğinden oynayan bir hikâyeyle ("Bir tur böyle işler") anlatılır.
- **Arayüz ↔ kontrat uyumu:** Kontrat ve arayüz paralel geliştirildi. Arayüz, kontratın gerçek yeteneklerini zincirden okuyup uyumsuz sürümde (sponsorlu, kurasız) işlem göndermeyi reddeder. Canlı kontrata karşı testte stub'ın göstermediği iki şey bulundu ve düzeltildi: SDK'nın `Result` sarmalayıcısı (`Ok { value }`) ve kontratın `get_member_status.refundable` değerinin tamamlanmış havuzun son turunda da dolu dönmesi (para satıcıya gittiği için arayüz bunu sıfır gösterir; `claim_refund` yalnızca iptal edilmiş havuzda çalışır).
- **Trade-off:** Kontrat yükseltilemez (yeni ID gerekir); zaman aşımı kendiliğinden işlem başlatmaz, herkes ilgili fonksiyonu çağırır.

## Teslim durumu

- [x] Sponsorsuz kontrat (API v11: kura + 30 üye + peşinat), 33 birim testi, Testnet'te yayında ve canlı senaryolar koşuldu (kura + peşinat tam akış, iptal + iade, kurulamayan havuz)
- [x] Kontrat ID ve dağıtım kanıtı (yukarıda)
- [x] Arayüz kontrata bağlı: okuma canlı, `create_pool` simüle edildi, kura arayüzde seçilebilir
- [x] Gerçek SEP-1/10/24 anchor istemcisi (test anchor'ı ile)
- [x] Mainnet için işlemsiz, canlı ağ bilgisini okuyan tanıtım derlemesi
- [x] Mimari diyagram ve teknik belgeler
- [ ] Cüzdan imzalı yazma akışının (havuz kur/öde/onayla/kura çek) uçtan uca denemesi
- [x] Herkese açık demo URL'i (https://stellerpool.arslanyusuf.com)
- [ ] **Gerçek TL anchor giriş/çıkışı ve kullanılabilir bakiye** (hackathon çekirdek gereksinimi, karşılanmıyor)
- [x] Kura modu ve 30 üye ([görev listesi](docs/CONTRACT_HANDOFF.md) uygulandı)
- [ ] Sunum (resmi Stellar şablonu)
- [ ] AI denetçisi için gerekçeli rapor ve insan kararı

## Yol haritası (hackathon sonrası)

1. Commit-reveal veya harici kaynakla doğrulanabilir kura rastgeleliği; gerçek 30 kişilik yük denemesi.
2. Doğrulanmış TRY anchor'ı ile gerçek TL giriş/çıkışı; çekme (Stellar → TRY) akışı.
3. Passkey / akıllı cüzdan ile kripto bilgisi olmayan kullanıcı için giriş.
4. AI destekli belge ve koşul tutarsızlığı raporu (fon yetkisi olmadan).
5. Gerçek ürün için lisanslı ortaklık ve hukuki çerçeve (BDDK/SPK/TCMB değerlendirmesi).

Bu belge ürün/teknik açıklamadır; hukuki görüş veya gerçek para güvencesi değildir.
