# Gerçek cüzdanla uçtan uca test kontrol listesi

Amaç: `https://stellerpool.arslanyusuf.com` üzerinde, gerçek bir Freighter cüzdanıyla,
kontrat + (varsa) anchor akışının uçtan uca gerçekten çalıştığını kanıtlamak — pitch/README
için ekran görüntüsü/video kanıtı burada toplanır. Bu, `docs/HACKATHON_SUBMISSION_TASKS.md`
madde 3'ün "Gerçek cüzdanla uçtan uca tam döngü" maddesidir.

## Üye sayısı ve süreler — ikisi de oluşturma formunda ayarlanabilir

Havuz oluşturma sayfası (`CreateView.vue`) hem üye sayısını hem dört süreyi (katkı, ek
süre, alım onayı, kuruluş) **serbestçe ayarlanabilir** yapar — sabit değildir:

- **Üye sayısı**: `+`/`−` düğmeleriyle **2 ile 30** arasında herhangi bir değer.
- **Süreler**: "Süreleri tek tek ayarla" bağlantısına tıklayınca dört ayrı açılır liste
  çıkar (katkı/ek süre/alım onayı/kuruluş), her birinde bir **"(demo)"** etiketli hızlı
  seçenek (katkı: 3 dakika, ek süre: 10 dakika, alım onayı: 30 dakika, kuruluş: 1 saat)
  ve gerçekçi uzun seçenekler (1 saat/1 gün/7 gün/30 gün) var.

**Hangi değerleri seçmeli — amaca göre:**
- **Bu kontrol listesini hızlı test etmek için** (mutlu yol, deadline beklemeden):
  tüm süreler için **"(demo)"** seçeneklerini kullanın — kimse ödemeyi geciktirmezse
  zaten hiçbir deadline'ı beklemeniz gerekmez, akış saniyeler içinde biter.
- **Canlı jüri sunumu için**: süreleri **uzun tutun** (en az "1 saat") — "(demo)"
  presetindeki 3 dakikalık katkı süresi, sunum sırasında konuşurken **ortasında
  dolabilir** ve havuz beklenmedik şekilde "gecikti" durumuna düşebilir. Alternatif:
  havuzu sunumdan önce siz oluşturun, sahnede yalnızca kalan adımları (katkı/kura/alım
  onayı) gösterin.
- **Gecikme/ek süre/iptal senaryosunu göstermek için** (isteğe bağlı, README'de zaten
  #7 ile kanıtlı — tekrar canlı göstermeniz şart değil): en kısa "(demo)" süreleri
  seçip gerçekten bekleyin — Soroban deadline'ları simüle edilemez, gerçek zaman geçmesi
  gerekir.
- **Üye sayısı**: kontrol listesini hızlı bitirmek için 2-3 yeterli; kura modunun
  "kazananın bir daha çıkmaması" davranışını daha çarpıcı göstermek isterseniz 4-5 üye
  deneyebilirsiniz (README'deki havuz #5 zaten 3 üyeyle kura modunu kanıtlamış durumda).

## Ön hazırlık

- [ ] [Freighter](https://www.freighter.app/) tarayıcı eklentisini kur, **Testnet** ağına
  geçir (eklenti ayarlarından).
- [ ] Seçtiğiniz üye sayısı kadar + 2 doğrulayıcı hesabı oluştur (Freighter'da birden
  fazla hesap eklenebilir) — doğrulayıcılar üye olamaz, ayrı hesap olmalı. Örn. 2 üyeli
  bir havuz için en az 4 hesap (2 üye + 2 doğrulayıcı).
- [ ] Her hesabı [Friendbot](https://lab.stellar.org/account/fund?$=network$id=testnet)
  ile fonla (XLM için).
- [ ] Havuz varlığı için (site hangi varlığı kullanıyorsa — `TRYT` veya `USDC`) her üye
  hesabına yeterli bakiye sağla: ya anchor akışıyla (aşağıda), ya doğrudan bir trustline +
  transfer ile.

## A) Temel havuz akışı (kontrat)

- [ ] 1. Kurucu hesabıyla siteye bağlan, yeni havuz oluştur: üye sayısını `+`/`−` ile
  seç, katkı tutarını gir, sıra modu = **Sabit** veya **Kura** seç, "Süreleri tek tek
  ayarla"ya tıklayıp yukarıdaki tabloya göre uygun süreleri seç.
- [ ] 2. Cüzdan onayı penceresini gerçekten imzala — `create_pool` işleminin
  stellar.expert'te göründüğünü doğrula (site genelde bir explorer linki gösterir).
- [ ] 3. İkinci hesapla havuza katıl (`join_pool`).
- [ ] 4. Kurucu, sırayı (Sabit modda) veya sadece doğrulayıcıları (Kura modda) ve
  doğrulayıcı listesini öner (`propose_terms`).
- [ ] 5. Her iki üye de şartları onayla (`approve_terms`) — iki ayrı cüzdan imzası.
- [ ] 6. Havuzu başlat (`start_pool`) — herhangi bir hesap tetikleyebilir.
- [ ] 7. Her iki üye de katkısını yatırsın (`deposit`) — ikinci yatırımdan sonra turun
  faza geçtiğini gözlemle (Sabit'te doğrudan "alım bekleniyor", Kura'da "kura bekleniyor").
- [ ] 8. **Kura modundaysa**: herhangi bir hesapla kura çek (`draw_recipient`) — kazananın
  ekranda göründüğünü doğrula.
- [ ] 9. Alıcı (kazanan/sıradaki üye) bir satın alma öner (`propose_purchase`) — havuzun
  izinli demo satıcısına, doğru tutar ve bir belge özetiyle.
- [ ] 10. Her iki doğrulayıcı da onaylasın (`approve_purchase`) — iki ayrı cüzdan imzası.
- [ ] 11. Turu tamamla (`execute_round`) — satıcının bakiyesinin arttığını, turun
  "tamamlandı" olduğunu doğrula.
- [ ] 12. **Kanıt**: her adımdaki işlem hash'lerini/stellar.expert linklerini not al —
  README'nin "Kontrat ve dağıtım kanıtı" bölümüne veya pitch'e eklenebilir.

## B) Anchor akışı (varsa siteye bağlandıysa — `docs/HACKATHON_SUBMISSION_TASKS.md` madde 2/3)

- [ ] 1. Havuz sayfasında katkı adımına gel, "Bakiyen yetmiyor mu? Anchor ile yükle"
  bölümünü aç.
- [ ] 2. SEP-10 girişini onayla (Freighter'da bir imza isteği çıkmalı — bu bir ödeme
  değil, kimlik doğrulama imzasıdır).
- [ ] 3. Açılan interaktif pencerede bir tutar gir, "TRY yatırdım, onayla (test)" butonuna
  bas. **Bu gerçek bir banka işlemi değildir** — sayfa bunu zaten açıkça yazar.
- [ ] 4. Eğer hesapta `TRYT` için trustline yoksa durum "trustline bekleniyor" gösterir —
  Freighter'dan bu varlık için bir "add trust line" işlemi onayla.
- [ ] 5. Birkaç saniye içinde bakiyenin otomatik güncellendiğini doğrula (sayfa
  durumu poll'luyor) — gerçek bir on-chain ödeme aldığını stellar.expert'te kontrol et.
- [ ] 6. Bu yüklenen bakiyeyle A) akışındaki katkı adımını (adım 7) tamamla.

## Kanıt toplama

Her adımda:
- Ekran görüntüsü (özellikle: cüzdan imza penceresi, başarı bildirimi, stellar.expert
  işlem sayfası) veya kısa bir ekran kaydı.
- İşlem hash'lerinin bir listesi (README/pitch'e eklemek için).

## Bir şey patlarsa

- Kontrat tarafı hata verirse: hata mesajını ve hangi adımda olduğunu not alıp kontrat
  ekibine ilet (`docs/HACKATHON_SUBMISSION_TASKS.md` madde 1'de bu triyaj görevi zaten
  var).
- Anchor tarafı hata verirse: `backend/README.md` "Uç noktalar" ve "Bilinçli
  basitleştirmeler" bölümlerine bakılabilir; sunucunun ayakta olduğunu `/health` ile
  doğrula.
