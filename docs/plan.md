# Stellerpool — davetli tasarruf grubu planı

**Amaç:** Tanıdık kişilerin altın günü gibi sabit sıralı katkılarını görünür kurallarla koordine eden Stellar prototipi. Hedef Soroban kontratında kurucunun ortak parayı serbestçe çekme yetkisi olmayacak; katkı, sıra ve ödeme koşulları üyelerce onaylanacak. Bu teknik sınır, erken tahsisat alan üyenin gelecek katkılarını garanti etmez.

**Aşama (19 Eylül 2026):** Lisans başvurusu veya gerçek müşteri fonu yok. Mainnet sitesi salt okunur tanıtımdır; cüzdan imzası, fon ve havuz işlemi yoktur. Testnet havuz kontratı henüz yazılıp yayınlanmadı. Anchor yalnızca etiketli simülasyon; gerçek TRY veya kullanılabilir bakiye yaratmaz. [Yayın kapsamı](mainnet-showcase.md) ve [hukuki sınır](altin-gunu-legal-boundary.md) ayrı kaydedildi. Hackathon DOC'unun gerçek TL giriş/çıkış ölçütü karşılanmıyor.

## 1. Eminevim ve Fuzul ile karşılaştırma

Her iki şirketin açıklamasında katkılar bir araya getirilir; çekilişli veya teslim tarihi belirli modeller vardır. Fuzul, grup dağılsa da kendi organizasyonunun taahhüdün arkasında olduğunu, ödeme dondurulursa teslimatın ertelendiğini ve teslim edilen ev/araç için ipotek/rehin kurulduğunu açıklıyor. Eminevim de ipotek teminatı, teslimat öncesi taksit dondurma/teslimat erteleme ve sözleşmeye dayalı tahsisat süreci anlatıyor. BDDK lisanslı şirketin sözleşme, ayrılmış fon havuzu ve müşteri hakları çerçevesini açıklıyor. **Bu açıklamalar ayrı bir sponsor cüzdanı modeli tarif etmiyor; yükümlülük şirketin hukuki ve mali yapısında.** Kamuya açık sayfalar teslimat sonrası her gecikmenin iç tahsilat adımlarını açıklamıyor.

| İhtiyaç | Testnet hedefi | Sınır |
|---|---|---|
| Kurucunun fonu alıp kaçamaması | Kurucuda serbest çekim fonksiyonu yok; ödeme yalnız kayıtlı tura/satıcıya | Kontrat, anahtar ve varlık ihraççısı riski sürer |
| Sıra ve koşulların değişmemesi | Tüm üyeler aynı sürümü onaylar; başlangıçtan sonra kilitlenir | Cüzdan adresleri gerçek kimliği kanıtlamaz |
| Eksik ödeme | Katkı son tarihi, ek süre ve turun durması | Gelecek ödemeler ve teslimat garanti edilmez |
| İptal/iade | Yalnız henüz satıcıya gitmemiş turun katkıları sahiplerine geri verilir | Önceki turda harcanmış katkılar kontrattan iade edilemez |
| Alım doğrulaması | Test satıcısı, belge özeti ve insan doğrulayıcı onayı | Gerçek tapu, ruhsat veya satıcı kimliği doğrulanmaz |

## 2. Sponsorsuz ürün kararı

1. Kapalı ve sabit üyeli grup; demo için 2–12 üye, kişi başına her tur aynı `C` katkısı ve `N` tur. Her üye bir kez sıradaki alıcıdır. Kura ve değişken tutar MVP dışında.
2. Kurucu yalnızca önerir. Katkı, sıra, süreler, satıcı ve doğrulayıcı koşulları üyelerin aynı sürümü onaylamasıyla geçerli olur. Kurucu tek başına değiştiremez veya fon çekemez.
3. Ayrı sponsor, sponsor güvencesi, başkası adına avans ve platformun teslimat garantisi yoktur. Fon eksiği yeni kullanıcıların katkısıyla kapatılmış varsayılmaz.
4. Her turda **bütün üyeler kendi katkılarını** yatırmadan tahsisat açılmaz. Son tarih geçince ek süre başlar; eksik üye bu sürede ödeyebilir. Hâlâ eksikse herkes havuzu sonlandırabilir.
5. Duran turda yalnız bu turda birikmiş ve henüz satıcıya ödenmemiş katkılar sahiplerine iade edilir. Daha önce tamamlanıp satıcıya ödenen turların katkısı geri çağrılamaz. Tahsisat almış üyelerin sonraki ödeme yükümlülüğü gerçek dünyada ancak ayrı sözleşme ve yetkili süreçlerle takip edilebilir; demo bunu garanti etmez.
6. Demo tahsisatı yalnız önceden belirlenmiş test satıcısına, alıcının önerisi ve insan doğrulayıcı eşiği sonrasında yapılır. Alım için ayrı son tarih vardır; öneri/onay yetişmezse mevcut tur katkıları iade edilir.
7. Ortak fon lending, staking veya likidite havuzuna gönderilmez. DeFi niteliği programlanabilir fon akışı ve doğrulanabilir kurallardır. AI denetçisi planlanan yardımcı rapordur; fon tutmaz veya tahsisat açmaz.

**Açık ekonomik risk:** Dört üye her tur 10 birim yatırırsa ilk turda 40 birim A'nın satıcısına çıkar. İkinci tur başlamadan B, C ve D'nin önceki 10'ar birimi kontratta değildir. A sonraki ödemeyi bırakırsa ikinci tur durabilir; B, C ve D'nin ilk tur ödemeleri otomatik iade edilemez. Üye sayısını büyütmek bu açığı yok etmez. Tam iade veya belirli tarihte teslimat sözü bu modelin dışında ayrı mali/hukuki kaynak ve yükümlülük gerektirir.

## 3. Ödeme aksadığında

| Durum | Testnet akışı | Sınır |
|---|---|---|
| Alıcı henüz teslim almadı, katkıyı geciktirdi | Tur ek süreye geçer; ödeme gelmezse bu tur iptal edilir | Teslimat tarihi garanti edilmez |
| Daha önce teslim alan kişi sonraki katkıyı geciktirdi | Yeni tur durur; yalnız o turda yatırılmış katkılar iade edilir | Önceki turdaki açığı kontrat kapatamaz; tahsilat ayrı süreçtir |
| Satıcı veya belge onayı yetişmedi | Mevcut tur satıcıya aktarılmadan durur ve tur katkıları iade edilir | Gerçek mülkiyet/ödeme senkronizasyonu yoktur |

Fuzul'ün taksit dondurma ve ipotek/rehin açıklaması kendi lisanslı ürününe ilişkindir. Bizim demo bu sözleşme ve teminatı sunmaz. Gerçek ürün yolunda [lisanslı ortaklık ve AI rolü](legal-ai-path.md) ayrıca değerlendirilir.

## 4. Kullanıcı akışı

~~~mermaid
flowchart TD
  A[Kurucu havuzu açar] --> B[Üyeler katılır ve koşulları onaylar]
  B --> C{Kuruluş süresinde grup tamam mı?}
  C -->|Hayır| X[Başlamadan iptal]
  C -->|Evet| D[Herkes havuzu başlatabilir]
  D --> E[Her üye kendi tur katkısını yatırır]
  E --> F{Tüm katkılar hazır mı?}
  F -->|Hayır, süre doldu| G[Ek süre]
  G -->|Ödeme geldi| F
  G -->|Ödeme gelmedi| R[Bu turun katkılarını iade et]
  F -->|Evet| H[Alıcı demo satıcısını ve belge özetini önerir]
  H --> I{İnsan onayı ve alım süresi uygun mu?}
  I -->|Hayır| R
  I -->|Evet| J[Bu turun tutarı satıcıya gider]
  J --> K{Son tur mu?}
  K -->|Hayır| E
  K -->|Evet| L[Havuz tamamlanır]
~~~

Zaman aşımı kendiliğinden zincir işlemi başlatmaz; süre dolunca herkes ilgili fonksiyonu çağırabilir. Tamamlanmış bir turun ödemesi geri alınamaz.

## 5. Hedef kontrat ve değişmezler

**Durumlar:** Havuz `Filling → Active → Completed / Aborted`. Tur `Collecting → Grace → AwaitingPurchase → Settled` olabilir. Katkı son tarihi ve ek süre ilk belirlenen zamanlara bağlıdır; gecikmiş çağrı süreyi uzatmaz. Tüm katkılar geldiğinde alım için ayrı süre başlar; satıcı önerisini yenilemek bu süreyi sıfırlamaz.

**P0 fonksiyonlar**
- `create_pool`: test varlığı, katkı, üye sınırı, süreler, kuruluş son tarihi ve izinli demo satıcısı. Ekonomik koşullar oluşturulduktan sonra değişmez.
- `join_pool`, `propose_terms`, `approve_terms`, `start_pool`, `cancel_unstarted_pool`: üyelik, sürümlü ortak onay ve kuruluş son tarihi. Tüm üyeler onaylamadan başlanmaz; kurucu çevrim dışı olsa da koşullar tamamsa herkes başlatabilir.
- `deposit`, `cure_payment`: yalnız ilgili üye kendi tur katkısını bir kez yatırır. Başkası adına ödeme veya avans modeli yoktur.
- `propose_purchase`, `approve_purchase`: alıcı izinli demo satıcısını, tutarı ve belge özetini önerir; onaylar havuz/tur/alıcı/satıcı/tutar/belge/sürümüne bağlıdır. Yeni öneri eski onayları siler.
- `execute_round`: bütün üye katkıları, geçerli doğrulayıcı eşiği, alım süresi ve satıcının varlığı alabilmesi doğrulanır; yalnız o turda toplanan tutar satıcıya çıkar.
- `mark_overdue`, `abort_pool`, `claim_refund`: ek süre veya alım süresi dolunca havuz sonlandırılabilir. İade hakkı yalnız henüz ödenmemiş **mevcut turun** bizzat yatırılmış katkısıdır; aynı iade iki kez alınamaz.
- `get_pool`, `get_round`, `get_member_status`: salt okunur durum.

**Değişmezler**
- Her havuz ve tur ayrı muhasebeleştirilir; başka havuzun varlığı kullanılamaz. Kontratın toplam bakiyesi bir turun tahsisat tutarı sayılmaz.
- Bir tur satıcıya ödenmeden önce gereken tutar `N × C` ve bütün üyeler `paid` olarak kayıtlı olmalıdır. Transfer ve tur durum değişimi atomiktir.
- İptalde mevcut turdaki üyelerin iade toplamı, o turun kontratta tuttuğu bakiyeye eşittir. Önceki tamamlanmış turlara iade hakkı yazılmaz.
- Kurucuya serbest çekim, yöneticinin tek taraflı sıra/satıcı değiştirme veya tek taraflı kod yükseltme yetkisi verilmez. Onaylar yalnız geçerli sürüm için sayılır.
- Kuruluş, katkı, ek süre, alım ve iade erişimi için Soroban TTL/arşiv davranışı tasarlanıp test edilir. Tutarlar en küçük birimde tamsayı olarak tutulur.

**Anlamlı testler:** İlk ve son turun tam akışı; daha önce tahsisat alan üyenin sonraki turda ödememesi; mevcut tur katkılarının tam iadesi ve geçmiş turun iade edilmemesi; aynı turda çift ödeme/iade; farklı havuzların bakiyeleri; eksik üyelik veya onay; satıcı değişince onayın sıfırlanması; satıcının trustline/izin eksikliği; süre aşımı; kurucu çevrim dışıyken başlatma; TTL. Kontrat henüz bulunmadığından bu testler yapılmış sayılmaz.

## 6. Zincir dışı parçalar ve hukuki sınır

- **Anchor:** Arayüzde yalnız simülasyon var; banka, TRY varlığı, kullanılabilir bakiye veya cüzdan imzası yok. Gerçek sağlayıcı/izinler ayrı doğrulanmadan hackathon'un TL şartı karşılandı denmez.
- **Ev/araç:** Demo satıcısı test cüzdanıdır. Tapu, ruhsat, satıcı kimliği, rehin/ipotek ve alacak tahsilatı kontratla yapılmaz. Gerçek ürün belirli teslimat vaadi sunacaksa sözleşme tarafı, fon ve ödeme akışı için lisanslı yapı ve yetkili süreç gerekir.
- **AI:** Yalnız yardımcı risk raporu; insan incelemesi ve kesin kontrat kuralları geçerlidir. Modelin çıktısı sözleşmeyi resmîleştirmez.
- **Kullanıcı verisi:** Gerçek müşteri verisi Testnet demo belgesine veya zincire konmaz.

## 7. Demo ve teslim durumu

1. Dört test cüzdanı; her tur 10 test birimi. Tüm üyeler sırayı ve kuralları onaylar.
2. İlk tur dört katkıyla 40 birim demo satıcısına gider; işlem bağımsız ağ gezgininde gösterilir.
3. İkinci turda ilk teslim alan üye ödemez. Ek süre sonunda yalnız ikinci turda yatırılan katkılar iade edilir; ilk tur katkılarının geri alınamadığı açık gösterilir.
4. Alternatif akışta geciken üye ek sürede kendi payını yatırır; ikinci tur ancak bütün katkılar tamamlanınca ilerler.
5. Kontrat ID, işlem bağlantıları ve yukarıdaki testler olmadan zincir üstü havuz demosu tamamlandı denmez. Bugünkü Mainnet tanıtımı işlemsizdir.

## 8. Açık kararlar

1. Grup içindeki gerçek alacak ve uyuşmazlıklar hangi sözleşme/kanalda yürütülecek? Uygulama bunu garanti etmeyecek.
2. Gerçek satıcı ve mülkiyet doğrulamasını hangi yetkili taraf yapacak?
3. Seçilen anchor gerçek TRY'yi hangi ağ, varlık ve koşullarla destekliyor?
4. Gerçek ev/araç finansmanı istenirse hangi lisanslı şirket sözleşme ve fon sorumlusu olacak?
5. Havuz başına ayrı kontrat mı, tek kontratta ayrık muhasebe mi kullanılacak?
6. Token/TL kuru ve satın alma gücü riski kullanıcıya nasıl anlatılacak?

## Kaynaklar

- [Eminevim Merak Edilenler](https://kiosk.eminevim.com.tr/merak-edilenler)
- [Fuzul Merak Edilenler](https://www.fuzulev.com.tr/merak-edilenler)
- [BDDK Tasarruf Finansman SSS](https://www.bddk.org.tr/Sss/Liste/117)
- [6361 sayılı Kanun](https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=6361&MevzuatTur=1&MevzuatTertip=5)
- [Stellar Asset Contract](https://developers.stellar.org/docs/tokens/stellar-asset-contract)

Kaynaklar 19 Eylül 2026 tarihinde yeniden incelendi. Bu belge ürün/teknik planıdır; hukuki görüş veya gerçek para güvencesi değildir.
