import card_parser
import pytest

@pytest.mark.parametrize("url, expected_result", [
("https://www.example.com", "https://www.example.com"),
("http://subdomain.example.com", "http://subdomain.example.com"),
("www.example.com", "www.example.com"),
("example.com", "example.com"),
("https://www.example.com/", "https://www.example.com/"),
("https://www.example.com/path/to/resource", "https://www.example.com/path/to/resource"),
("https://www.example.com/path/to/resource/", "https://www.example.com/path/to/resource/"),
("https://www.example.com?param=value", "https://www.example.com?param=value"),
("https://www.example.com#section", "https://www.example.com#section"),
("ISBPN https://www.example.com/path/to/resource?param=value#section", "https://www.example.com/path/to/resource?param=value#section"),
("ISBPN https://www.example.com/path/to/resource#section?param=value", "https://www.example.com/path/to/resource#section?param=value"),
("", None),
("https://", None),
("http://", None),
("ffdafdaf https://www.example.com?param=value&param2=value2 21 123jlkj kjkj", "https://www.example.com?param=value&param2=value2"),
("https://www.example.com/path/to/resource?param=value&param2=value2 fdafdaslfkj lkf lk ", "https://www.example.com/path/to/resource?param=value&param2=value2"),
("ISBPMhttps://www.example.com", "https://www.example.com"),
("http://192.168.0.1/", "http://192.168.0.1/"),
("http://localhost:3000 yo", "http://localhost:3000"),
("HIhttp://localhost:3000", "http://localhost:3000"),
("https://www.google.com/search?q=fd&hl=en&authuser=0&tbm=isch&sxsrf=AB5stBhrlS4x6FpVzM4feqrqMOxJSjihgg%3A1689787808101&source=hp&biw=1584&bih=800&ei=oB24ZKOoBOma0-kPl7-D2AQ&iflsig=AD69kcEAAAAAZLgrsNjM_BPkfiZDD1gRameSDW75ElSK&ved=0ahUKEwij-K6QppuAAxVpzTQHHZffAEsQ4dUDCAc&uact=5&oq=fd&gs_lp=EgNpbWciAmZkMgQQIxgnMggQABiABBixAzIIEAAYgAQYsQMyBRAAGIAEMggQABiABBixAzIFEAAYgAQyBRAAGIAEMggQABiABBixAzIFEAAYgAQyCBAAGIAEGLEDSMcCUABYR3AAeACQAQCYAcABoAHdAqoBAzAuMrgBA8gBAPgBAYoCC2d3cy13aXotaW1n&sclient=img", "https://www.google.com/search?q=fd&hl=en&authuser=0&tbm=isch&sxsrf=AB5stBhrlS4x6FpVzM4feqrqMOxJSjihgg%3A1689787808101&source=hp&biw=1584&bih=800&ei=oB24ZKOoBOma0-kPl7-D2AQ&iflsig=AD69kcEAAAAAZLgrsNjM_BPkfiZDD1gRameSDW75ElSK&ved=0ahUKEwij-K6QppuAAxVpzTQHHZffAEsQ4dUDCAc&uact=5&oq=fd&gs_lp=EgNpbWciAmZkMgQQIxgnMggQABiABBixAzIIEAAYgAQYsQMyBRAAGIAEMggQABiABBixAzIFEAAYgAQyBRAAGIAEMggQABiABBixAzIFEAAYgAQyCBAAGIAEGLEDSMcCUABYR3AAeACQAQCYAcABoAHdAqoBAzAuMrgBA8gBAPgBAYoCC2d3cy13aXotaW1n&sclient=img"),
])
def test_remove_extra_trailing_chars(url, expected_result):
    assert card_parser.remove_extra_trailing_chars(url) == expected_result
