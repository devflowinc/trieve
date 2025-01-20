# CrawlOptions

Options for setting up the crawl which will populate the dataset.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allow_external_links** | **bool** | Option for allowing the crawl to follow links to external websites. | [optional] 
**body_remove_strings** | **List[str]** | Text strings to remove from body when creating chunks for each page | [optional] 
**boost_titles** | **bool** | Boost titles such that keyword matches in titles are prioritized in search results. Strongly recommended to leave this on. Defaults to true. | [optional] 
**exclude_paths** | **List[str]** | URL Patterns to exclude from the crawl | [optional] 
**exclude_tags** | **List[str]** | Specify the HTML tags, classes and ids to exclude from the response. | [optional] 
**heading_remove_strings** | **List[str]** | Text strings to remove from headings when creating chunks for each page | [optional] 
**ignore_sitemap** | **bool** | Ignore the website sitemap when crawling, defaults to true. | [optional] 
**include_paths** | **List[str]** | URL Patterns to include in the crawl | [optional] 
**include_tags** | **List[str]** | Specify the HTML tags, classes and ids to include in the response. | [optional] 
**interval** | [**CrawlInterval**](CrawlInterval.md) |  | [optional] 
**limit** | **int** | How many pages to crawl, defaults to 1000 | [optional] 
**scrape_options** | [**ScrapeOptions**](ScrapeOptions.md) |  | [optional] 
**site_url** | **str** | The URL to crawl | [optional] 
**webhook_metadata** | **object** | Metadata to send back with the webhook call for each successful page scrape | [optional] 
**webhook_url** | **str** | Host to call back on the webhook for each successful page scrape | [optional] 

## Example

```python
from trieve_py_client.models.crawl_options import CrawlOptions

# TODO update the JSON string below
json = "{}"
# create an instance of CrawlOptions from a JSON string
crawl_options_instance = CrawlOptions.from_json(json)
# print the JSON string representation of the object
print(CrawlOptions.to_json())

# convert the object into a dict
crawl_options_dict = crawl_options_instance.to_dict()
# create an instance of CrawlOptions from a dict
crawl_options_form_dict = crawl_options.from_dict(crawl_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


