# Gerçek cüzdanla uçtan uca test kontrol listesi

Amaç: `https://stellerpool.arslanyusuf.com` üzerinde, gerçek bir Freighter cüzdanıyla,
kontrat + (varsa) anchor akışının uçtan uca gerçekten çalıştığını kanıtlamak — pitch/README
için ekran görüntüsü/video kanıtı burada toplanır. Bu, `docs/HACKATHON_SUBMISSION_TASKS.md`
madde 3'ün "Gerçek cüzdanla uçtan uca tam döngü" maddesidir.

## Ön hazırlık

- [ ] [Freighter](https://www.freighter.app/) tarayıcı eklentisini kur, **Testnet** ağına
  geçir (eklenti ayarlarından).
- [ ] En az **3 farklı hesap** oluştur (Freighter'da birden fazla hesap eklenebilir) —
  bir tanesi kurucu/üye, ikincisi ikinci üye, üçüncüsü doğrulayıcı rolü için. 2 üyeli
  bir havuzda en az 2 üye + 2 doğrulayıcı hesabı gerekir (doğrulayıcılar üye olamaz).
- [ ] Her hesabı [Friendbot](https://lab.stellar.org/account/fund?$=network$id=testnet)
  ile fonla (XLM için).
- [ ] Havuz varlığı için (site hangi varlığı kullanıyorsa — `TRYT` veya `USDC`) her üye
  hesabına yeterli bakiye sağla: ya anchor akışıyla (aşağıda), ya doğrudan bir trustline +
  transfer ile.

## A) Temel havuz akışı (kontrat)

- [ ] 1. Kurucu hesabıyla siteye bağlan, yeni havuz oluştur (üye sayısı, katkı tutarı,
  sıra modu = **Sabit** veya **Kura** seç, süreler).
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
