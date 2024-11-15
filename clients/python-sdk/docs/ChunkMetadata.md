# ChunkMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** | HTML content of the chunk, can also be an arbitrary string which is not HTML | [optional] 
**created_at** | **datetime** | Timestamp of the creation of the chunk | 
**dataset_id** | **str** | ID of the dataset which the chunk belongs to | 
**id** | **str** | Unique identifier of the chunk, auto-generated uuid created by Trieve | 
**image_urls** | **List[Optional[str]]** | Image URLs of the chunk, can be any list of strings. Used for image search and RAG. | [optional] 
**link** | **str** | Link to the chunk, should be a URL | [optional] 
**location** | [**GeoInfo**](GeoInfo.md) |  | [optional] 
**metadata** | **object** | Metadata of the chunk, can be any JSON object | [optional] 
**num_value** | **float** | Numeric value of the chunk, can be any float. Can represent the most relevant numeric value of the chunk, such as a price, quantity in stock, rating, etc. | [optional] 
**tag_set** | **List[Optional[str]]** | Tag set of the chunk, can be any list of strings. Used for tag-filtered searches. | [optional] 
**time_stamp** | **datetime** | Timestamp of the chunk, can be any timestamp. Specified by the user. | [optional] 
**tracking_id** | **str** | Tracking ID of the chunk, can be any string, determined by the user. Tracking ID&#39;s are unique identifiers for chunks within a dataset. They are designed to match the unique identifier of the chunk in the user&#39;s system. | [optional] 
**updated_at** | **datetime** | Timestamp of the last update of the chunk | 
**weight** | **float** | Weight of the chunk, can be any float. Used as a multiplier on a chunk&#39;s relevance score for ranking purposes. | 

## Example

```python
from trieve_py_client.models.chunk_metadata import ChunkMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkMetadata from a JSON string
chunk_metadata_instance = ChunkMetadata.from_json(json)
# print the JSON string representation of the object
print(ChunkMetadata.to_json())

# convert the object into a dict
chunk_metadata_dict = chunk_metadata_instance.to_dict()
# create an instance of ChunkMetadata from a dict
chunk_metadata_form_dict = chunk_metadata.from_dict(chunk_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


