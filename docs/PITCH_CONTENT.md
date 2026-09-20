# Sunum içeriği taslağı (Pro Hackathon 2026)

**Sürüm notu:** Bu taslak tarihsel v10/v11 demo kanıtları içerir. Yeni doğrulayıcısız API v12 dağıtılana kadar canlı sitede yeni akış gösterilemez. Güncel ürün akışı için [plan](plan.md) esas alınır.

Bu, resmi Stellar sunum şablonuna (kopyasını alıp doldurun — orijinali düzenlemeyin)
yapıştırmaya hazır, slayt slayt içerik taslağıdır. Şablonun kendi slayt sırası/başlıkları
bundan farklıysa, içeriği ona göre dağıtın — yapı değil, içerik önemli.

**Dil notu:** Aşağıdaki metin Türkçe yazıldı (projenin çalışma dili). Jüri karışık/
uluslararası olabileceğinden sunumu İngilizce'ye çevirmek muhtemelen daha güvenli —
bu bir karar noktası, siz seçin.

---

## 1. Başlık / Takım

**Stellarpool** — şeffaf, akıllı sözleşme tabanlı grup tasarruf havuzu (Stellar/Soroban)

> Birlikte biriktir. Her şeyi doğrula.

Track: **Genesis**

---

## 2. Problem (Narrative Why)

Türkiye'de ev, araç ya da ortak bir hedef için grup halinde birikim yapmak çok
yaygın: altın günleri, tasarruf finansman şirketlerinin (Eminevim, Fuzul gibi)
çekilişli/sıralı grupları.

**Ortak sorun güven:**
- Para bir kişinin ya da bir kurumun elinde toplanır.
- Kurallar (kimin ne zaman alacağı, biri ödemezse ne olacağı) genelde şeffaf değil.
- Katılımcının elinde, "param nerede, kurallar gerçekten uygulanıyor mu" sorusuna
  cevap verecek bir kanıt yok — güvene dayalı bir sistem.

**Hedef kullanıcı:** birbirini tanıyan, ortak bir alım için sırayla ya da kurayla
birikim yapmak isteyen küçük gruplar (arkadaş çevresi, aile, iş arkadaşları).

---

## 3. Çözüm / Değer önerisi

Stellarpool bu koordinasyonu bir kişiye ya da kuruma değil, **bir Soroban akıllı
sözleşmesine** emanet eder:

- Katkılar tur bazında doğrudan kontratta toplanır — kurucunun ortak parayı tek
  başına çekme yetkisi **hiçbir zaman** yoktur.
- API v12 akışında ödeme, **tüm üyelerin onayladığı kurallar**, eksiksiz tur katkıları ve alıcının alım kaydı sonrasında kayıtlı demo satıcısına gider — alıcının cebine değil.
- Alıcı sırası ya da **kura** ile belirlenir (30 üyeye kadar grup desteklenir).
- Bir ödeme aksarsa tur durur; yalnızca **o turun** katkıları sahiplerine iade
  edilir — kurallar kod içinde, herkes zincir üstünden doğrulayabilir.

Bu, Fuzul/Eminevim gibi mevcut ürünlerin sunduğu deneyime (sıra/kura ile ev-araç
birikimi) yaklaşırken, "güven bana" yerine "doğrula" diyen bir model sunuyor.

---

## 4. Canlı demo

- **Demo URL:** https://stellerpool.arslanyusuf.com (Testnet, canlı kontrata bağlı)
- **Kontrat:** `CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB` (API v10,
  Stellar Testnet)
- V12 dağıtıldıktan sonra gösterilecek akış: plan gir → uygun havuza katıl veya aç → üyeler koşulları onaylar → başlat → katkı yatır → (kura varsa) kura çek → alımı kaydet → satıcıya öde.

*(Buraya 2-3 ekran görüntüsü veya kısa bir GIF/video ekleyin: havuz oluşturma,
kura çekme anı, tamamlanmış havuz.)*

---

## 5. Teknik mimari

**Akıllı sözleşme (Soroban, Rust):**
- `rotating_pool` kontratı — çok havuzlu, tam durum makinesi:
  `Filling → Active → Completed | Aborted`; tur: `Collecting → Grace →
  (AwaitingDraw) → AwaitingPurchase → Settled`.
- 4 nesil boyunca gerçek üretim baskısıyla evrildi: sponsor modelinden
  sponsorsuz modele, ardından kura modu + 30 üyeye — her adımda **canlı Testnet
  deploy'u ve gerçek işlem kanıtı** ile (`docs/IMPLEMENTATION_LOG.md`, 14 faz).
- **27 birim testi**, 30 üyeli tam kura akışı dahil; her invocation Mainnet kaynak
  limitlerine (`InvocationResourceLimits::mainnet()`) karşı otomatik doğrulanıyor.
- Checks-effects-interactions deseni, `require_auth` ile rol bazlı yetkilendirme,
  TTL yönetimli persistent storage.

**Anchor (SEP-1/10/24) — kendi backend'imiz:**
- `testanchor.stellar.org`'a bağımlı kalmak yerine **kendi SEP-1/10/24 servisimizi**
  yazdık (`backend/`, Node.js/TypeScript).
- Gerçek bir Testnet varlığı (`TRYT`, TRY'yi temsil eder) ihraç edildi; SEP-10
  cüzdan girişi, SEP-24 interaktif yatırma, `pending_trust` durumunu kendiliğinden
  yeniden deneyen bir akış — uçtan uca, gerçek Testnet işlemleriyle doğrulandı.
- Anchor artık ayrı bir vitrin değil, **havuza katkı adımının içinde** ("Bakiyen
  yetmiyor mu? Anchor ile yükle").

**Frontend:** Vue 3 + TypeScript, Stellar Wallets Kit ile gerçek cüzdan bağlantısı,
kontratın zincirdeki arayüzünü okuyup kura/kapasite gibi özellikleri otomatik
açıp kapatan bir yetenek-algılama katmanı.

---

## 6. Stellar ekosistem uyumu

- **Entegrasyon partneri:** Stellar Wallets Kit (cüzdan bağlantısı, uygulamanın
  her işleminde çekirdek bir parça).
- **Anchor / yerel ödeme:** kendi SEP-1/10/24 servisimiz, gerçek protokol akışı
  (test varlığıyla — gerçek TRY/banka entegrasyonu değil, bu açıkça belirtiliyor).
- **Kullanılan Stellar Skills** *(tam liste ve gerekçeleri: `docs/STELLAR_SKILLS_USED.md`)*:
  - `skills.stellar.org/skills/smart-contracts/SKILL.md`
  - `skills.stellar.org/skills/dapp/SKILL.md`
  - `skills.stellar.org/skills/assets/SKILL.md`
  - `skills.stellar.org/skills/standards/SKILL.md` (SEP seçimi)
  - `skills.stellar.org/skills/data/SKILL.md`
  - Anchors skill (SEP-1/6/10/12/24/31/38) — SEP-10/24 sunucu implementasyonunun
    doğrudan referansı.

---

## 7. Bilinen sınırlar (dürüstçe söylenmeli)

- Gerçek, lisanslı bir TRY anchor'ı henüz yok — bu bilinçli bir kapsam kararı,
  sonraki adımın ne olduğu net (bkz. madde 8).
- Erken alıcının (sıra ya da kurayla) sonraki turda ödemeyi bırakması, önceki
  turun katkılarını geri getirmez — bu, ürünün kabul ettiği bir ekonomik risk,
  grup büyüklüğü bunu çözmüyor. Şeffaf şekilde belgelendi (`docs/plan.md`).
- Kura modunda zincir üstü rastgelelik hackathon düzeyinde (Soroban PRNG,
  validator etkisine açık) — gerçek kullanım için commit-reveal gerekir.

---

## 8. Traction & sonraki adım (roadmap)

- 4 fazlık gerçek iterasyon: sponsor modelden sponsorsuz modele, kuraya ve
  30 üyeye — her biri canlı Testnet kanıtıyla belgelendi.
- Kendi anchor'ımızı kurup uçtan uca doğrulamak, "gerçek TL girişi" hedefine
  somut bir adım attı (test varlığıyla, ama gerçek protokolle).
- **Sonraki adım:** gerçek, lisanslı bir TRY anchor sağlayıcısıyla görüşmek (SEP-24
  uyumlu), KYC/AML entegrasyonu, mainnet'e geçiş, SCF/InstaAward başvurusu.

---

## 8b. Anchor hakkında olası jüri soruları (hazırlık notu, sunuma girmez)

⚠️ **Sunumdan hemen önce kontrol edin**: canlı site (`stellerpool.arslanyusuf.com`)
şu an kendi backend'imize mi bağlı, yoksa hâlâ SDF'in genel test anchor'ına mı
düşüyor (`frontend/src/lib/anchor.ts`'deki `usingTestAnchor` bayrağı bunu söyler —
arayüzde muhtemelen bir rozet/uyarı olarak görünür). Bu belgenin yazıldığı an
Yusuf'un sunucu deploy'u (`backend/DEPLOY.md`) henüz tamamlanmamıştı; tamamlandıysa
aşağıdaki "kısa cevap" kullanılabilir, tamamlanmadıysa "henüz bağlanmadı" cevabını
kullanın — ikisi de dürüst ve savunulabilir, önemli olan hangisinin doğru olduğunu
bilip söylemek.

**Kısa cevap (backend canlı siteye bağlandıysa):**
> Kendi SEP-1/10/24 anchor sunucumuzu yazdık — SDF'in genel test anchor'ına bağımlı
> kalmak yerine. Gerçek protokol akışı çalışıyor: SEP-10 girişi, SEP-24 interaktif
> yatırma, TRY'yi temsil eden kendi test varlığımız (`TRYT`) gerçek on-chain
> işlemlerle kullanıcıya ulaşıyor. Bunu hem kendi başına hem kontratımızla (gerçek
> bir havuz üzerinden) uçtan uca, gerçek Testnet işlemleriyle doğruladık. Gerçek
> olmayan tek kısım: "TRY yatırdım" onayı bir bankadan gelmiyor, sunucumuzun kendi
> test onayı — gerçek banka entegrasyonu bilinçli olarak kapsam dışı, yol
> haritamızın ilk adımı bu.

**Eğer backend henüz canlı siteye bağlanmadıysa:**
> Backend kodu yazıldı ve gerçek Testnet işlemleriyle (iki ayrı uçtan uca akış,
> ayrıca kontratımızla birlikte) doğrulandı — [tx hash göster]. Canlı siteye
> bağlanması operasyonel son adım, üzerinde çalışıyoruz.

**Olası takip soruları:**

| Soru | Cevap |
|---|---|
| Bu gerçek TL mi? | Hayır. `TRYT` TRY'yi temsil eden bir test varlığı. Gerçek TL, lisanslı bir anchor/banka entegrasyonu gerektirir — bilinçli olarak kapsam dışı, sonraki adımımız bu. |
| Neden SDF'in test anchor'ını kullanmadınız? | El kitabı Anchor'ın ürünün çekirdeğinde olmasını ve en ağırlıklı kriter olduğunu belirtiyor. Genel bir üçüncü taraf test anchor'ına bağlanmak bunu göstermiyordu; kendi SEP altyapımızı kurup uçtan uca kanıtlamak daha güçlü bir teknik kanıt. |
| Hangi SEP'leri destekliyor? | SEP-1 (stellar.toml), SEP-10 (kimlik doğrulama), SEP-24 (interaktif yatırma). SEP-12/KYC ve withdraw kapsam dışı, yol haritasında. |
| Kontratla nasıl bağlanıyor? | Anchor'dan gelen bakiye havuzun kullandığı varlığın aynısı (kod + ihraççı eşleşmesi kontrol edilir) — kullanıcı önce yatırır, sonra o bakiyeyle doğrudan havuza katkı sağlar; ayrı bir vitrin değil, katkı adımının içinde. |
| Hangi Stellar Skill'i kullandınız? | Anchors skill (SEP-1/6/10/12/24/31/38) — SEP-10/24 sunucu implementasyonumuzun doğrudan referansı; tam liste `docs/STELLAR_SKILLS_USED.md`'de. |

## 9. Takım

*(İsimler, roller, iletişim bilgileri — submission formuyla aynı olmalı.)*

---

## 10. Kapanış / Sorular

**Stellarpool: "güvenmen gerekmiyor, doğrulayabiliyorsun."**
