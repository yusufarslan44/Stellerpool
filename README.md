# Stellerpool — şeffaf grup tasarruf havuzu (Stellar / Soroban)

> Birlikte biriktir. Her şeyi doğrula. — *Save together. Verify everything.*

Rise In x Stellar **Pro Hackathon 2026** projesi · Track: **Genesis** (teslimde seçilecek) · Ağ: **Stellar Testnet**

## Neden? (Narrative "Why")

Türkiye'de ev, araç ya da ortak bir hedef için grup halinde birikim yapmak yaygındır: altın günleri, tasarruf finansman şirketlerinin (Eminevim, Fuzul gibi) çekilişli ya da sıralı grupları. Ortak sorun **güvendir**: para bir kişinin ya da bir kurumun elindedir, kurallar herkese görünmez, biri ödemeyi bırakınca ne olacağı belirsizdir.

Stellerpool bu koordinasyonu **Soroban akıllı sözleşmesine** taşır. Katkılar tur bazında kontratta kilitlenir, kurucunun ortak parayı çekme yetkisi yoktur, ödeme yalnızca üyelerin onayladığı kurallar ve doğrulayıcı onayı sonrası, önceden belirlenmiş satıcıya gider. Ödeme aksarsa tur durur ve yalnızca o turun katkıları sahiplerine iade edilir. **Hedef kullanıcı:** birbirini tanıyan, ortak bir alım için sırayla ya da kurayla birikim yapmak isteyen küçük gruplar.

## Bugün ne çalışıyor, ne çalışmıyor?

| Alan | Durum |
|---|---|
| Soroban kontratı (sponsorsuz, API v9) | ✅ Testnet'te yayında, 19 birim testi, canlı senaryo koşuldu |
| Arayüz ↔ kontrat | ✅ Okuma yolu canlı kontrata karşı doğrulandı (havuz, tur, üye). ⚠️ Cüzdan imzalı yazma akışı (havuz kur, öde, onayla) bir cüzdanla uçtan uca henüz denenmedi |
| Anchor | ✅ Gerçek **SEP-1 / SEP-10 / SEP-24** istemcisi (SDF test anchor'ı ile doğrulandı). ❌ **Türk lirası değil**: test anchor'ı yalnızca test varlığı üretir. Cüzdan imzalı yatırma penceresi bir cüzdanla denenmedi |
| Kura modu ve 30 üye | 🟡 Arayüz hazır, kontrat henüz desteklemiyor (kontrat: 12 üye, sabit sıra). [Kontrat görev listesi](docs/CONTRACT_HANDOFF.md) |
| Mainnet | Salt okunur tanıtım derlemesi; cüzdan imzası veya fon işlemi yok |
| Gerçek TL giriş/çıkışı | ❌ **Hackathon'un çekirdek gereksinimi karşılanmıyor**, doğrulanmış bir TRY anchor'ı bulunamadı ([ayrıntı](docs/altin-gunu-legal-boundary.md)) |
| AI denetçisi | ❌ Çalışan entegrasyon yok, yol haritasında |

## Kontrat ve dağıtım kanıtı (Testnet)

- **Kontrat ID:** [`CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH`](https://stellar.expert/explorer/testnet/contract/CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH) · API sürümü 9 · WASM SHA-256 `8535d21f…5b2f`
- **Yükleme:** [`83bbb089…456c`](https://stellar.expert/explorer/testnet/tx/83bbb0895b7a7952d5347603b296c7dfd3845444e3c53940682e1a76426a456c) · **Örnek oluşturma:** [`730248d5…3d97`](https://stellar.expert/explorer/testnet/tx/730248d5e027d91422114620df41351dc7348e30177db3685e9fc45889783d97)
- **Canlı senaryo (havuz #1):** [`create_pool`](https://stellar.expert/explorer/testnet/tx/486f97b0d56b08ddb5473586b63ea235add8bbd98dca1c2a6a060ec03200f2dc) → [`start_pool`](https://stellar.expert/explorer/testnet/tx/0d829021ab4cdec22ba824c23bff9fd7fdb93e4e3627f1576dd2f3d45a310abf) → [`mark_overdue`](https://stellar.expert/explorer/testnet/tx/9e2db4eec262987cdf77f3c4d79b470f8aa77f95b8ab45ec324533ff1b497ba1) → `cure_payment` → [`execute_round`](https://stellar.expert/explorer/testnet/tx/fcf9190504f4a9d0a1c84a3d32cfa217ed977f5feab9160eb99c9215564513ab) → 2. turda ödeme yok → [`mark_overdue`](https://stellar.expert/explorer/testnet/tx/05838e108c3555e1986ec0ae9a6ccec8c728583a828ca10e98f175953c8e5b7f) → [`abort_pool`](https://stellar.expert/explorer/testnet/tx/3ccb735023d0a124f42bbf624fd94bb0f94e3e7f31c6ba83e4196a2d78610cb9) → `claim_refund`
- Demo varlığı: `STLP` (SAC `CAOV35NPIJXHWA7QPXXERRJQ4ZTDUGAEHA7FKB6QIOTIOTI62B35ZNWI`). Arayüz varsayılan olarak Testnet USDC ile çalışır (kontrat varlığı havuz başına verilir).
- Önceki iki (sponsorlu) örnek de zincirde durur; teslimde yalnızca yukarıdaki güncel kontrat geçerlidir. Ayrıntı: [IMPLEMENTATION_LOG.md](docs/IMPLEMENTATION_LOG.md) "Phase 12".
- **Demo URL:** henüz yayınlanmadı (barındırma bekleniyor).

## Havuz kuralı

- Davetli, sabit üyeli grupta katkı, takvim ve sıra üyelerin aynı sürümü onaylamasıyla kilitlenir. Kurucu tek başına değiştiremez, fon çekemez.
- Her turda herkes **kendi** katkısını yatırmadan o turun tahsisatı açılmaz. Ödeme gecikirse ek süre verilir; karşılanmazsa tur durdurulur.
- O tur henüz satıcıya ödenmediyse yalnızca **o turda yatırılan katkılar** iade edilir. Satıcıya gitmiş katkılar kontrattan geri alınamaz.
- Demo tahsisatı yalnızca izinli test satıcısına, alım önerisi ve doğrulayıcı eşiği (2/3) onayı sonrasında gider. Alım için ayrı son tarih vardır.
- Ayrı sponsor, avans veya platform garantisi yoktur.
- **Hedef:** sıra yerine **kura** (her tur, henüz teslim almamış üyeler arasından, tüm katkılar tamamlanınca) ve **30 üyeye kadar** grup.

**Açık ekonomik risk:** Dört üye 10'ar birim yatırıp ilk turda 40 birim A'nın satıcısına ödenirse havuzda ilk turun parası kalmaz. A sonraki turda ödemezse yeni tur durur; B, C ve D'nin ilk tur katkıları kontrattan geri alınamaz. Üye sayısını büyütmek bunu çözmez. Bu risk arayüzde ve hikâye bölümünde görünür bırakıldı. [Ayrıntı](docs/plan.md).

## Fuzul Ev / Oto ve Eminevim ile karşılaştırma

| | Tasarruf finansman şirketleri | Stellerpool (Testnet prototipi) |
|---|---|---|
| Fon | Şirketin ayrılmış fon havuzu | Soroban kontratında tur bazında kilitli, kurucu çekemez |
| Alıcı | Sıra veya çekiliş | Onaylanan sabit sıra (kura hedefte) |
| Kurallar | Sözleşme, şirkete bağlı | Herkese görünür, zincirde doğrulanabilir |
| Ödeme aksarsa | Taksit dondurma, teslim erteleme | Ek süre → tur durur → mevcut tur iadesi |
| Teslim sonrası güvence | İpotek/rehin, şirket taahhüdü | **Yok** (açık risk) |
| Lisans | BDDK lisanslı | **Yok**, Testnet, gerçek para yok |

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

**Arayüz:** `cd frontend && npm ci`, `.env` içine `VITE_ROTATING_POOL_CONTRACT_ID=<yukarıdaki ID>` yazın, `npm run dev`. Mainnet'e bağlanan işlemsiz tanıtım derlemesi için `VITE_STELLAR_NETWORK=mainnet npm run build`. İsteğe bağlı: `VITE_ANCHOR_HOME_DOMAIN` (varsayılan SDF test anchor'ı), `VITE_MAX_MEMBERS` (varsayılan 12).
**Kontrat:** `cargo test --workspace`; Testnet dağıtımı için [scripts/README.md](scripts/README.md).

## Stellar entegrasyonları

- **Soroban akıllı kontrat** (Rust): havuz, tur ve iade muhasebesi, doğrulayıcı eşiği.
- **Stellar Wallets Kit**: cüzdan bağlama ve imza.
- **Anchor (SEP-1/10/24)**: stellar.toml keşfi, cüzdan imzalı SEP-10 girişi, SEP-24 interaktif yatırma. Test anchor'ı ile doğrulandı; TRY sağlayıcısı henüz yok.
- **Stellar Asset Contract (SAC)**: havuz varlığı (Testnet USDC ya da demo varlığı).
- **Stellar SDK 17 / Soroban RPC / Horizon**: sözleşme istemcisi ve hesap okumaları.

## Tasarım kararları ve çözülen zorluklar

- **Sponsor kaldırıldı:** İlk tasarım sponsor güvencesi içeriyordu; "para zaten kilitli" sezgisiyle çeliştiği için ve gerçek riski gizlediği için kaldırıldı. Yerine riski açıkça gösteren sponsorsuz model geldi.
- **Yalnızca mevcut tur iade edilir:** Satıcıya ödenmiş turun parası kontratta değildir. Bu kısıt saklanmak yerine arayüzde etkileşimli bir hikâyeyle ("Bir turu kendin dene") anlatılır.
- **Arayüz ↔ kontrat uyumu:** Kontrat ve arayüz paralel geliştirildi. Arayüz, kontratın gerçek yeteneklerini zincirden okuyup uyumsuz sürümde (sponsorlu, kurasız) işlem göndermeyi reddeder. Canlı kontrata karşı testte SDK'nın `Result` sarmalayıcısı gibi stub'ın göstermediği hatalar bulundu ve düzeltildi.
- **Trade-off:** Kontrat yükseltilemez (yeni ID gerekir); zaman aşımı kendiliğinden işlem başlatmaz, herkes ilgili fonksiyonu çağırır.

## Teslim durumu

- [x] Sponsorsuz kontrat (API v9), 19 birim testi, Testnet'te yayında ve canlı senaryo koşuldu
- [x] Kontrat ID ve dağıtım kanıtı (yukarıda)
- [x] Arayüz kontrata bağlı, okuma yolu canlı doğrulandı
- [x] Gerçek SEP-1/10/24 anchor istemcisi (test anchor'ı ile)
- [x] Mainnet için işlemsiz, canlı ağ bilgisini okuyan tanıtım derlemesi
- [x] Mimari diyagram ve teknik belgeler
- [ ] Cüzdan imzalı yazma akışının (havuz kur/öde/onayla) uçtan uca denemesi
- [ ] Herkese açık demo URL'i
- [ ] **Gerçek TL anchor giriş/çıkışı ve kullanılabilir bakiye** (hackathon çekirdek gereksinimi, karşılanmıyor)
- [ ] Kura modu ve 30 üye (kontrat işi, [görev listesi](docs/CONTRACT_HANDOFF.md))
- [ ] Sunum (resmi Stellar şablonu)
- [ ] AI denetçisi için gerekçeli rapor ve insan kararı

## Yol haritası (hackathon sonrası)

1. Kura ve 30 üyeye ölçekleme; commit-reveal ile doğrulanabilir rastgelelik.
2. Doğrulanmış TRY anchor'ı ile gerçek TL giriş/çıkışı; çekme (Stellar → TRY) akışı.
3. Passkey / akıllı cüzdan ile kripto bilgisi olmayan kullanıcı için giriş.
4. AI destekli belge ve koşul tutarsızlığı raporu (fon yetkisi olmadan).
5. Gerçek ürün için lisanslı ortaklık ve hukuki çerçeve (BDDK/SPK/TCMB değerlendirmesi).

Bu belge ürün/teknik açıklamadır; hukuki görüş veya gerçek para güvencesi değildir.
