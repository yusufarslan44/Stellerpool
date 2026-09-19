# Teslim öncesi görev listesi — Pro Hackathon 2026

**Kaynak:** bu belge, projenin Pro Hackathon 2026 el kitabına karşı yapılan uyumluluk
analizinin (bkz. konuşma geçmişi / `docs/IMPLEMENTATION_LOG.md`) doğrudan sonucu.
Amaç: kalan işi **frontend / backend / kontrat** arasında net, çakışmasız, kabul
ölçütlü görevlere bölmek.

## ⚠️ Zaman uyarısı

Teslim son tarihi **20 Eylül 2026, 12:00**. Bu belgenin yazıldığı an "bugün" zaten
20 Eylül 2026 — yani kalan süre saatler mertebesinde olabilir, gün değil. Aşağıdaki
her bölüm **P0 (teslim için zorunlu, saatler içinde yapılabilir)**,
**P1 (zaman kalırsa)**, **P2 (hackathon sonrası yol haritası, şimdi yapma)** olarak
etiketlenmiştir. Gerçek kalan süre netleşince P0 listesi bile sığmıyorsa, sıradan
düşün: önce P0'ın en üstündeki maddeler, geri kalanı pitch'te "sıradaki adım" olarak
anlatılır — jüri kriteri 5 (Traction & Continuity) zaten "credible roadmap" arıyor,
her şeyin bitmiş olmasını değil.

## 0. Durum özeti (güncellendi — bkz. Phase 13/14, `docs/IMPLEMENTATION_LOG.md`)

- **Kontrat**: v10 (kura modu + 30 üye) Testnet'te yayında, 27 test yeşil, canlı
  senaryo kanıtlı. ✅ Tamam.
- **Backend/Anchor**: kendi SEP-1/10/24 servisimiz yazıldı, gerçek `TRYT` varlığı
  ihraç edildi, uçtan uca (iki ayrı gerçek Testnet işlemiyle) doğrulandı. ✅ Kod tamam,
  ⚠️ şu an yalnızca geçici bir tünelde çalışıyor — kalıcı sunucu deploy'u (madde 2)
  bekleniyor.
- **Frontend**: v10'a bağlandı, anchor'ı havuz akışına gömdü (ayrı vitrin değil artık),
  **canlı demo URL'i yayında**: `https://stellerpool.arslanyusuf.com`. ✅ Büyük ölçüde
  tamam — kalan: bu sitenin anchor ortam değişkenlerini kalıcı backend'e çevirmek ve
  gerçek cüzdanla uçtan uca denemek (madde 3).
- **README**: kontrat ID'si v10, demo URL'i ekli. ✅ Tamam.
- **Hâlâ eksik**: kalıcı (sunucuda) anchor deploy'u, frontend'in ona bağlanması, gerçek
  cüzdan testi, resmi sunum şablonu. Aşağıdaki maddeler bunlara odaklanıyor.

## 1. Kontrat ekibi — kalan iş azalan öncelikte

Kontrat tarafı fonksiyonel olarak tamam; buradaki iş artık **diğer ekiplere
destek** ve küçük doğrulamalar.

**P0**
- [ ] Backend'in ihraç edeceği TRY-temsili test varlığının (bkz. madde 2) kontratla
  uyumunu doğrula: `create_pool`'a `token` olarak bu yeni SAC adresi verildiğinde
  `deposit`/`execute_round` sorunsuz çalışıyor mu — tek bir CLI denemesi yeterli
  (kontrat zaten herhangi bir SAC ile çalışacak şekilde asset-agnostik yazıldı, bu
  sadece bir doğrulama, kod değişikliği beklenmiyor).
- [ ] Frontend'in gerçek cüzdanla uçtan uca denemesi sırasında (madde 3) ortaya
  çıkabilecek kontrat-kaynaklı hataları hızlı triyaj et (varsa).

**P1**
- Yok — kapsam genişletmeye gerek yok, kalan saatler diğer eksik alanlara gitmeli.

**P2 (hackathon sonrası)**
- `docs/CONTRACT_HANDOFF.md`'nin "Bilinen sınırlar" bölümünde zaten yazılı: gerçek
  rastgelelik (commit-reveal), erken alıcı temerrüdü riski — bunlar bilinçli kapsam
  dışı, şimdi dokunma.

## 2. Backend ekibi — sıfırdan kuruldu (Phase 14)

Şu an proje backend'siz. "Anchor" dediğimiz şey frontend'in doğrudan
`testanchor.stellar.org`'a konuştuğu bir istemci — kendi sunucumuz yok. El kitabının
en ağırlıklı kriterini karşılamak için **kendi minimal SEP-1/10/24 anchor
servisimizi** kurmamız gerekiyor. Bu, gerçek bir banka entegrasyonu **değil** — SEP
protokolüne tam uyumlu ama "TRY yatırıldı" onayının simüle edildiği (admin onayı /
sabit gecikme) bir servis. Dürüstlük şart: hem kodda hem pitch'te "kendi işlettiğimiz
test anchor'ı, gerçek banka rayı yok" net yazılmalı — aksi hem etik hem
(`docs/altin-gunu-legal-boundary.md`'de zaten işaretlenmiş) hukuki risk yaratır.

**Durum:** temel servis yazıldı, gerçek Testnet varlığı ihraç edildi, uçtan uca
(SEP-10 login → SEP-24 interactive deposit → `pending_trust` → trustline → otomatik
tamamlanma → gerçek bakiye) **canlı test edildi ve doğrulandı**. Kod: `backend/`.
Ayrıntılı kanıt ve kararlar: `docs/IMPLEMENTATION_LOG.md` "Phase 14".

**P0 — minimum çalışan anchor**
- [x] Kendi issuer hesabını oluştur (Testnet), TRY'yi temsil eden bir test varlığı
  ihraç et. Kod adı net "test/temsili" olduğunu belli etsin — `TRYT`, `backend/scripts/setup-issuer.ts`.
- [x] SEP-1 `stellar.toml` servis et — `backend/src/routes/wellKnown.ts`.
- [x] SEP-10 auth endpoint — `backend/src/routes/auth.ts` (`WebAuth.buildChallengeTx`/
  `readChallengeTx`/`verifyChallengeTxSigners`, `frontend/src/lib/anchor.ts`'nin
  kullandığı aynı SDK modülü).
- [x] SEP-24 minimum uçlar + interaktif form — `backend/src/routes/sep24.ts`,
  `backend/src/routes/interactive.ts`. `pending_trust` durumu her poll'da kendiliğinden
  yeniden denenir.
- [x] `frontend/.env`'deki `VITE_ANCHOR_HOME_DOMAIN`'i https adresine çevir — **geçici
  olarak yapıldı**: `cloudflared` quick tunnel açıldı (hesap gerektirmedi), backend şu an
  `https://projection-democratic-leslie-orders.trycloudflare.com` üzerinden dışarı açık,
  `frontend/.env`'e yazıldı. ⚠️ Bu tünel yalnızca geliştiricinin makinesi açıkken çalışır,
  adres her yeniden başlatmada değişir — **teslim için kalıcı değil.**
- [ ] **Kalıcı sunucu deploy'u — Yusuf'un sunucusuna.** Adım adım talimat:
  [`backend/DEPLOY.md`](../backend/DEPLOY.md). Özet: aynı issuer/distribution sırlarını
  (güvenli kanaldan) sunucuya taşı — **yeniden `setup-issuer` çalıştırma**, bir alt alan
  adı + reverse proxy (Caddy/nginx) ile TLS, pm2/systemd ile sürekli ayakta tutma. Bu
  bitmeden teslim edilen link, sunucu değil geliştiricinin laptobuna bağımlı kalır.
- [x] **Kabul ölçütü** (backend tarafı için): `backend/scripts/test-flow.mjs` ile
  gerçek bir Testnet hesabı üzerinden tam SEP-10+SEP-24 döngüsü ve gerçek bir on-chain
  ödeme **hem localhost hem genel tünel adresi üzerinden** doğrulandı (iki ayrı tx hash,
  `docs/IMPLEMENTATION_LOG.md`'de). Gerçek bir cüzdanla (Freighter) tarayıcı üzerinden
  deneme henüz yapılmadı.

**P1 — zaman kalırsa**
- [ ] SEP-12 KYC-lite form (ad, e-posta — gerçek doğrulama yok, sadece akış tamlığı
  için).
- [ ] Withdraw yönü (kullanıcı `TRYT`'yi geri "TRY"ye çevirme simülasyonu).
- [ ] Deposit sonrası havuza otomatik "bakiye yükle" köprüsü (aşağıda frontend
  madde 3, P0'daki manuel bağlantının otomasyonu).

**P2 (hackathon sonrası, pitch'te yol haritası olarak anlat)**
- Gerçek bir lisanslı TRY anchor sağlayıcısıyla görüşme (SEP-24 uyumlu bir üçüncü
  taraf), gerçek banka/ödeme kuruluşu entegrasyonu, KYC/AML.

## 3. Frontend ekibi

**Durum:** Yusuf paralelde bu bölümün büyük kısmını bitirdi — kontrat v10'a bağlandı,
README güncellendi, **canlı bir demo URL'i yayında** (`https://stellerpool.arslanyusuf.com`),
ve anchor artık `PoolView.vue`'daki katkı adımının içinde ("Bakiyen yetmiyor mu? Anchor
ile yükle"), ayrı bir vitrin olmaktan çıktı. `frontend/src/lib/anchor.ts`'e eklenen
`supportsPoolAsset(code, issuer)` kontrolü, backend'in `stellar.toml`'undaki
`[[CURRENCIES]]` alanıyla (kod + ihraççı eşleşmesi) doğrulandı — **backend ile frontend
tarafı birbiriyle uyumlu**, kalan tek şey ikisini gerçek (kalıcı) bir adresle birbirine
bağlamak.

**P0**
- [x] Anchor'ı ana akışa bağla — tamamlandı (`e052ee9`, `PoolView.vue`).
- [x] Frontend'i public bir URL'e deploy et — tamamlandı (`https://stellerpool.arslanyusuf.com`).
- [x] README güncelle (kontrat ID v10, demo URL) — tamamlandı (`f878df5`, `1ce10b5`).
- [ ] **Canlı sitenin ortam değişkenlerini kalıcı anchor'a çevir**: `backend/DEPLOY.md`
  tamamlanınca (madde 2), `stellerpool.arslanyusuf.com`'un deploy ortamında
  `VITE_ANCHOR_HOME_DOMAIN=anchor.<domain>`, `VITE_POOL_ASSET_CODE=TRYT`,
  `VITE_POOL_ASSET_ISSUER=GAOPTL4Q34VQWE5PWWVYEX7QQLOSO2DVYUVURCHHXTZFKASS66YL7YJ3` ayarla
  ve yeniden deploy et. **Şu an site ya boş `VITE_ANCHOR_HOME_DOMAIN` (SDF test anchor'ı)
  ya da hiç ayarlanmamış anchor değişkenleriyle çalışıyor olabilir — kontrol edilmeli.**
- [ ] **Gerçek cüzdanla (Freighter) uçtan uca tam döngü**: havuz oluştur → üye
  katıl → şartları öner/onayla → başlat → öde → (kura modundaysa) kura çek →
  execute_round → tamamlandı, ve ayrıca yukarıdaki anchor bağlantısı gerçek bir cüzdanla
  denenmeli. Ekran görüntüsü veya kısa video al (pitch/README için kanıt).
- [ ] **Resmi Stellar sunum şablonunu kopyala ve doldur** (kopyasını al, orijinali
  düzenleme). Zorunlu teslim maddesi, henüz yapılmadı.

**P1 — zaman kalırsa**
- [ ] Kura/30 üye UI'ını canlı v10 kontratına karşı manuel dene (arayüz zaten zincirdeki
  kontrat yeteneğini okuyup kura seçeneğini otomatik açıyor — tek eksik gerçek bir
  denemenin yapılmış olması).
- [ ] `docs/architecture/architecture.md`'deki Anchor akış diyagramını, gerçek backend
  servisini yansıtacak şekilde güncelle (şu an SDF test anchor'ını varsayıyor).

**P2**
- Passkey/smart wallet desteği — el kitabı bunu açıkça "bonus, zorunlu değil" diyor,
  şimdi uğraşma.

## 4. Teslim portalı kontrol listesi (herkes, son adım)

- [ ] Takım adı + tüm üyelerin isim/iletişim bilgisi
- [x] GitHub repo linki: `github.com/yusufarslan44/Stellerpool`
- [x] Canlı demo URL'i: `https://stellerpool.arslanyusuf.com`
- [ ] Sunum linki, "Anyone with the link can view" olarak paylaşım ayarı açık
- [ ] Track seçimi: **Genesis**
- [x] Kontrat ID'si güncel (v10): `CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB`
- [ ] Demo sitesinin anchor bağlantısı kalıcı backend'e çevrildikten sonra siteyi son
  kez elle dene (madde 2 ve 3 bitince)
