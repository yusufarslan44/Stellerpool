# Stellerpool — korumalı ev/araç tasarruf havuzu planı

**Amaç:** FuzulEv/FuzulOto benzeri düzenli birikim ve sıralı teslimat deneyimini Stellar üzerinde, fonları organizatörün kullanımından ayırarak kurmak. Ana güven vaadi: organizatör ortak parayı kendi hesabına çekemez; her tahsisatın koşulları, alıcısı ve tutarı denetlenebilir.

**Aşama:** Testnet prototipi. Gerçek TL, gerçek ev/araç teslimi veya lisanslı tasarruf finansmanı hizmeti sunduğumuz iddia edilmeyecek.
**Hackathon:** Rise In x Stellar Pro Hackathon 2026, Genesis; teslim 20 Eylül 2026 12:00.

## 1. Referans model ve farkımız

Fuzul'ün resmi anlatımında kişiler ödeme güçlerine göre gruplara ayrılıyor; çekilişli ve teslim tarihi baştan belirlenen seçenekler bulunuyor. Teslimat sonrasında taksitler sürüyor. Fuzul, teslimat öncesinde taksit dondurmanın teslimatı da ertelediğini, grup dağılmasına karşı organizasyonun arkasında durduğunu ve teslim edilen ev/araç için ipotek veya rehin uyguladığını söylüyor. BDDK, ödemesini aksatan tasarruf dönemi müşterisinin tahsisatının sözleşmeye göre ertelenebileceğini veya sözleşmesinin feshedilebileceğini açıklıyor. Fuzul'ün kamuya açık SSS'si teslimat **sonrası** tahsilatın tüm adımlarını açıklamıyor. Bizim prototipimiz bu kurumsal, hukuki ve bilanço güvencelerinin yerine geçmez.

| İhtiyaç | Prototipteki karşılık | Açık sınır |
|---|---|---|
| Organizatör parayı alıp kaçamasın | Katkı ve sponsor güvencesi Soroban kontratında; creator'ın serbest çekim yetkisi yok | Kontrat hatası, anahtar ve varlık ihraççısı riski sürer |
| Sıra ve ödemeler değiştirilemesin | Grup başladıktan sonra üye, katkı, sıra ve süre kilitli; olaylar zincirde | Kura ve esnek ödeme planları MVP dışında |
| Erken teslim alan taksitleri bırakırsa diğerleri korunabilsin | Havuz başlamadan sponsorun ayrı varlığı kilitlenir; güvenli devam mümkün değilse bekleyenlere iade | Sponsor ekonomik kaybı üstlenir; gerçek dünyadaki tahsilat ayrı iştir |
| Tahsisat amaç dışına gitmesin | Alıcı, satıcı adresini ve alım kaydını önerir; doğrulama sonrası ödeme doğrudan satıcı cüzdanına | Tapu, ruhsat ve gerçek satıcı kimliği zincir dışında doğrulanır |
| TL ile kullanılabilsin | Anchor ile TL giriş/çıkışı araştırılır; kontrat yalnızca Stellar varlığı görür | Anchor, ihraççı ve bankacılık katmanı merkezi taraflardır |

## 2. Ürün kararı

1. **MVP: sabit sıralı, sponsor güvenceli havuz.** Üye başına bir teslimat ve her dönem bir katkı. Kura ve bireysel teslim tarihi varyantları sonraya bırakılır.
2. **Organizatör yalnızca kurulum yapar.** Başladıktan sonra sırayı, satıcıyı tek başına değiştiremez veya kontrat fonunu çekemez. Kontratta tek taraflı upgrade/kaçış kapısı bulunmaz.
3. **Katılımcıdan girişte bir taksitlik teminat alınmaz.** Eski plan bu teminatı yeterli koruma gibi sunuyordu; dört üyeli örnekte iki taksitlik açık bırakıyordu. Ayrıca tasarruf dönemindeki teminat kuralları hukuken incelenmelidir.
4. **Sponsor güvencesi ayrı tutulur.** Demo sponsoru Testnet varlığını yatırır. Gerçek üründe sponsorun kim olacağı, fonun niteliği ve düzenleyici statü hukuk ve iş ortaklığı kararıdır.
5. **Gecikme teslimat öncesi ve sonrası ayrı ele alınır.** Önce bildirim ve sözleşmede belirlenmiş ek süre gelir. Teslimat almamış üyenin kendi tahsisat hakkı ertelenebilir. Teslimat almış üyenin borcu sürer; sponsor açığı ayrıca yatırıp bekleyenleri koruyabilir. Güvenli devam sağlanamazsa ek süre sonunda iptal ve iade devreye girer. Başka üyelerin alacağı sessizce azaltılmaz.
6. **Tahsisat satıcıya yapılır.** Demo için test satıcısı cüzdanı kullanılır ve gerçek ev/araç satın alındığı söylenmez. Gerçek ürün, satıcı ve mülkiyet doğrulama ortağı gerektirir.
7. **Getiri stratejisi yok.** Ortak fonu lending, staking veya likidite havuzuna yatırmak MVP güven vaadini geniş risklere açar. DeFi özelliği programlanabilir saklama ve doğrulanabilir kurallardır.

### Ödeme aksadığında izlenecek yol

| Aşama | Fuzul/BDDK kaynaklarında doğrulanabilen | Stellerpool için karar |
|---|---|---|
| Teslimat öncesi | Fuzul taksit dondurmanın teslimatı ertelediğini söylüyor; BDDK sözleşmeye göre tahsisatın ötelenebileceğini veya sözleşmenin feshedilebileceğini açıklıyor. | Gecikme bildirimi ve ek süre; ödeme yapılmazsa ilgili üyenin tahsisatı durur. Küçük ve sabit sıralı MVP grubunda başka üyelerin sırası tek taraflı değiştirilmez. Üye değiştirme/yeni sıra gerçek ürünün ayrı tasarımıdır. |
| Teslimat sonrası | Fuzul teslim edilen ev/araç üzerinde ipotek veya rehin bulunduğunu söylüyor; kamuya açık SSS kesin bir ihtar, yapılandırma veya tahsilat takvimi vermiyor. | Borç kaydı ve bildirim; sözleşme ve lisanslı ortak varsa yapılandırma/tahsilat. Gerçek tahsisattan önce geçerli ipotek/rehin ve mülkiyet doğrulanır. Sponsor, tahsilatı beklerken diğer üyelerin iade hakkını karşılar. |
| İyileşme olmazsa | Gerçek dünyadaki alacak ve teminat işlemleri sözleşme ve hukuk yoluyla yürütülür. | Testnet'te ek süre sonunda sponsor katkı yapmazsa iptal/iade. Gerçek üründe alacağın takibi ile iptal/iade birbirinden ayrı yürür; kontrat mülke kendiliğinden el koyamaz. |

Bildirim, ek süre ve yeniden yapılandırma sırası **bizim ürün önerimizdir**; Fuzul'ün açıklamadığı iç tahsilat süreci olarak sunulmaz. Teslimat sonrası üyeye süre tanınması, sponsor finansmanı veya başka bir onaylı kaynak olmadan diğer üyelerin alacağını eksiltemez. Sponsorun eksik katkıyı yatırması borcu silmez; gerçek para sürümünde kime borç doğacağı sözleşmeyle belirlenir.

## 3. Ekonomik güvence hesabı

N üye, kişi başına her tur C katkı, N tur ve her tur bir tahsisat varsayılır. r tamamlanmış turdan sonra henüz tahsisat almamış N-r kişinin o ana kadar yatırdığı toplam tutar r × (N-r) × C'dir. Bu kişilerin havuz durursa yatırdıklarını geri alabilmesi için kontrat bakiyesi en az bu tutarı karşılamalıdır. Başlangıçta kilitlenecek sponsor güvencesinin alt sınırı, bu dizinin tepe noktası olan **floor(N²/4) × C**'dir. Bu formül yalnızca bu sabit, eşit katkılı model ve aşağıdaki iade kuralı içindir; gerçek varlık değeri, kur ve hukuki tahsilat riskini kapsamaz.

**Örnek:** Dört üye, her tur 10 birim öder. Sponsor başta 40 birim kilitler. İlk tur sonunda A'nın satıcısına 40 ödenir; B, C ve D'nin önceki toplam 30 birimi için kontratta 40 kalır. İkinci turda herkes öderse B'nin satıcısına 40 ödenir ve henüz teslim almamış C ile D'nin yatırdığı 40 birim kontratta kalır. İlk teslim alan A ikinci turda ödemezse tahsisat kendiliğinden gerçekleşmez: sponsor tamamlayıp iade yeterliliğini koruyabilir veya havuz iptal edilir ve bekleyenler yatırdıklarını kontrattan geri alır. Sponsor bu durumda kayıp yaşayabilir.

**İade hakkı:** İptalde teslimat almamış üyeler yaptıkları tüm katkıları; teslimat almış üyeler yalnızca henüz tamamlanmamış tur için yatırdıkları katkıyı geri alır. Tamamlanmış turların katkısı geri alınamaz. Sponsor, tüm üye iadelerinden sonra kalan güvenceyi alır. Aynı varlık için havuzlar arası borç/alacak mahsuplaşması yapılmaz.

## 4. Kullanıcı akışı

~~~mermaid
flowchart TD
  A[Havuz ve sabit ödeme planı oluştur] --> B[Sponsor güvencesini kontrata kilitle]
  B --> C[Üyeler cüzdanla katılır]
  C --> D[Sıra onaylanır ve havuz başlar]
  D --> E[Tur katkıları yatırılır]
  E --> F{Üye katkıları veya sponsor tamamlamasıyla tur tutarı hazır mı?}
  F -->|Evet| G[Alıcı satıcıyı ve alım kaydını önerir]
  G --> H[Bağımsız doğrulama ve imzalar]
  H --> I{Ödeme sonrası iade yeterliliği korunuyor mu?}
  I -->|Evet| J[Tutar doğrudan satıcıya gider]
  J --> K{Son tur mu?}
  K -->|Hayır| E
  K -->|Evet| L[Sponsor kalanı geri alır]
  F -->|Süre doldu| M[Tur durur, üye bilgilendirilir]
  M --> P[Ek süre ve teslimat öncesi/sonrası durumu]
  P --> N{Üye öder veya sponsor açık tutarı tamamlar mı?}
  N -->|Evet: ödeme kayda geçer| F
  N -->|Hayır| O[İptal ve hak sahiplerine iade]
  I -->|Hayır| M
~~~

Demo sırasında alım doğrulaması imzalı test verisidir; gerçek satıcı veya tapu entegrasyonu değildir. Kontrat fon transferini doğrular, fiziksel mülkiyet devrini kendi başına bilemez.

## 5. Kontrat tasarımı

**Durumlar:** Filling → Active → Grace → Paused → Completed / Aborted. Grace ve Paused durumlarında yeni tahsisat yapılamaz. Herkese açık çağrılar zamanı gelmiş işlemleri tetikler; zincirde işlemler kendi kendine çalışmaz.

**P0 fonksiyonlar**
- create_pool: varlık, katkı, üye sayısı, tur süresi ve sponsor adresini belirler.
- fund_guarantee: sponsor güvencesini ayrı muhasebe kaydına yatırır; yeterli güvence olmadan havuz başlamaz.
- join_pool: üyeyi kaydeder; cüzdan ve varlık uygunluğunu doğrular.
- start_pool: tüm üyeler, sıra ve güvence hazırsa planı kilitler.
- deposit: üyenin mevcut tur katkısını bir kez alır.
- propose_purchase: sıradaki üye satıcı adresi ve zincir dışı belge özetini kaydeder.
- approve_purchase: belirlenen doğrulayıcıların imzalarını kontrol eder; creator'ın tek başına onayı yeterli değildir.
- execute_round: üye katkıları veya kayda geçirilmiş sponsor tamamlamasıyla tur tutarı tam, alım onaylı ve ödeme sonrası iade yeterliliği sağlanıyorsa yalnızca o turun tutarını kayıtlı satıcıya yollar. Herkes çağırabilir.
- mark_overdue, cure_payment, top_up, abort_pool: süre aşımını ve ek süreyi işaretler; üye borcunu tamamlayabilir veya sponsor eksik katkıyı ilave fonla o tur için karşılayabilir. Ödeme yeterliyse durum yeniden Active olur. Aynı katkı ikinci kez tahsil edilmez. Ek süre bittiğinde güvenli devam yoksa herkes iptali tetikleyebilir. Sponsor ödemesi, teslimat almış üyenin zincir dışı borcunu otomatik silmez.
- claim_refund, claim_sponsor_remainder: iptal veya tamamlanma durumuna göre hak sahibine iade. İki kez talep engellenir.
- get_pool, get_round, get_member_status, get_refund_claim: okuma.

**Değişmezler**
- Hiçbir fonksiyon havuzun toplam kontrat bakiyesini doğrudan tahsisat tutarı saymaz.
- Her transfer belirli pool_id, tur ve hak sahibi ile ilişkilidir; başka havuzun varlığı kullanılamaz.
- Her tahsisat sonrasında kontratın o havuza atanan bakiyesi, teslimat almamış üyelerin tüm katkıları ile teslimat almış üyelerin henüz tamamlanmamış tur katkılarından doğan iade hakları toplamından az olamaz; aynı katkı iki kez sayılmaz.
- Sponsor güvencesi, üye alacakları kapatılmadan çekilemez.
- Aynı üye aynı turda ikinci kez ödeme veya aynı iade için ikinci kez talep yapamaz.
- Tur, başlama, ödeme sonu ve iptal bekleme zamanları zincirde saklanır; depolama TTL'si uzatılır.
- Varlık tutarları tamsayı en küçük birimle saklanır; arayüzde string/BigInt kullanılır.

**Güvenlik:** Yetkilendirme, taşma, yeniden giriş/çapraz kontrat çağrısı, çok havuzlu muhasebe, kötü niyetli satıcı, hatalı doğrulama, eksik ödeme, iade yarışı ve TTL test edilir. Gerçek fonlardan önce bağımsız kontrat denetimi gerekir.

## 6. Zincir dışı parçalar ve mevzuat

- **Anchor:** Gerçek TL → Stellar varlığı veya tersi için sağlayıcı ve SEP akışı workshop'ta doğrulanacak. SEP-24 tercih; desteklenmiyorsa SEP-6. Jüri şartının sandbox ile karşılanıp karşılanmadığı organizatöre sorulacak. Sağlayıcı netleşmeden gerçek TL desteği iddia edilmez.
- **Varlık:** Anchor'ın verdiği token, ihraççı, geri ödeme hakkı, freeze/clawback yetkileri ve kur riski gösterilecek. TL olmayan varlıkta taksit ile TL satın alma gücü aynı şey değildir.
- **Gerçek ev/araç:** Satıcı kimliği, fatura/sözleşme, tapu/ruhsat, teslimat öncesi gerekli ipotek/rehin tescili ve ihtilaflar zincir dışı doğrulayıcı veya lisanslı ortak gerektirir. Doğrulayıcı yanlış bilgi verirse kontratın doğru ödeme yapması tek başına kaybı önlemez. Kontrat, ihtar veya taşınmaz/araç üzerindeki hukuki tahsilatı yürütemez.
- **Hukuki model:** Tasarruf finansmanı faaliyeti, müşteri fonu, sponsor güvencesi, ipotek/rehin, KYC/AML ve ödeme hizmetleri yetkileri uzmanla ve yetkili kurumlarla netleştirilecek. Testnet demosu faaliyet izni anlamına gelmez. Katılımcıdan tasarruf döneminde nakit teminat alma fikri ayrıca değerlendirilmeden geri getirilmeyecek.
- **Çekiliş:** MVP'de yok. Gelecekte eklenirse doğrulanabilir rastgelelik, katılım ve iptal kuralları ayrıca tasarlanacak.

## 7. Hackathon demosu ve teslim ölçütü

1. Dört üye ve bir demo sponsoru Testnet cüzdanlarıyla katılır; örnek katkı 10 birim, sponsor güvencesi 40 birim.
2. Arayüz, havuz bakiyesini, sponsor güvencesini, bekleyen üyelerin iade hakkını, sırayı ve tur son tarihini ayrı gösterir.
3. Bir TL anchor akışı gerçekten çalışıyorsa en az bir TL giriş veya çıkışı gösterilir; sağlayıcı ve işlem kanıtı belgelenir.
4. İlk turda herkes öder; doğrulanmış demo satıcısına 40 birim gider. Zincir işlemi ve satıcı adresi gösterilir.
5. İkinci turda daha önce tahsisat alan A ödemez; ödeme süresi dolunca tur durur, gecikme ve ek süre gösterilir. A ödemez ve sponsor tamamlamazsa ek süre sonunda iptal tetiklenir; teslimat almamış üyeler katkılarını geri alır ve sponsorun zararı görünür.
6. Pozitif ikinci senaryoda A ek sürede öder veya sponsor açık tutarı ekler; iade yeterliliği korunuyorsa tur tamamlanır. Sponsor ödemesi A'nın borcunu silmiş gibi gösterilmez.
7. Testler: normal tur, farklı havuzların ayrılığı, eksik katkı, sahte satıcı/onay, yetersiz güvence, iptal/iade, çift talep ve muhasebe değişmezi.
8. Testnet contract ID, kurulum adımları, demo URL, gerçek/simüle edilen parçaların sınırları ve sunum hazırlanır.

**Öncelik:** Önce fon muhasebesi ve iptal/iade; ardından satıcıya ödeme, cüzdan, anchor ve arayüz. Hukuken doğrulanmamış gerçek para veya gerçek ev/araç teslim vaadi yapılmaz.

## 8. Açık kararlar

1. Sponsor kim olacak ve gerçek üründe kaybı hangi bilanço taşıyacak?
2. Satıcı ve mülkiyet doğrulamasını hangi yetkili taraf yapacak? Sahte satıcı ve danışıklı işlem nasıl engellenecek?
3. Seçilen anchor gerçek TL'yi hangi ağda, hangi varlık ve minimum tutarla destekliyor?
4. Düzenlenmiş tasarruf finansmanı kapsamına giren faaliyetler için lisanslı ortak mı, başka bir hukuki yapı mı gerekiyor?
5. Havuz başına ayrı kontrat mı kullanılacak? Tek kontrat, ayrı muhasebeye rağmen ortak kod riski taşır.
6. Üyelerin para birimi TL ise token/TL kur değişimi ve satın alma gücü riski kime ait?

## 9. Araştırma kaynakları

- Fuzul sistemin işleyişi: https://www.fuzulev.com.tr/nasil-calisir
- Fuzul SSS (teslimat sonrası taksit, grup güvencesi, ipotek/rehin): https://www.fuzulev.com.tr/merak-edilenler
- BDDK tasarruf finansman SSS (gecikme, tahsisat erteleme, fon ayrımı ve müşteri hakları): https://www.bddk.org.tr/Sss/Liste/117
- BDDK lisanslı şirket listesi: https://www.bddk.org.tr/Kurulus/Liste/89
- Stellar Asset Contract ve ihraççı yetkileri: https://developers.stellar.org/docs/tokens/stellar-asset-contract

Kaynaklar 19 Eylül 2026 tarihinde incelendi. Bu belge ürün ve teknik planıdır; hukuki görüş veya gerçek para güvencesi değildir.
