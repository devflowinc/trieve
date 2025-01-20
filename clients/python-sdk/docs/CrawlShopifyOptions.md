# CrawlShopifyOptions

Options for Crawling Shopify

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_variants** | **bool** | This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product. default: true | [optional] 
**tag_regexes** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.crawl_shopify_options import CrawlShopifyOptions

# TODO update the JSON string below
json = "{}"
# create an instance of CrawlShopifyOptions from a JSON string
crawl_shopify_options_instance = CrawlShopifyOptions.from_json(json)
# print the JSON string representation of the object
print(CrawlShopifyOptions.to_json())

# convert the object into a dict
crawl_shopify_options_dict = crawl_shopify_options_instance.to_dict()
# create an instance of CrawlShopifyOptions from a dict
crawl_shopify_options_form_dict = crawl_shopify_options.from_dict(crawl_shopify_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


