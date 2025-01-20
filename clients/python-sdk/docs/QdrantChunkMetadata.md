# QdrantChunkMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** |  | [optional] 
**dataset_id** | **str** |  | 
**group_ids** | **List[str]** |  | [optional] 
**image_urls** | **List[str]** |  | [optional] 
**link** | **str** |  | [optional] 
**location** | [**GeoInfo**](GeoInfo.md) |  | [optional] 
**metadata** | **object** |  | [optional] 
**num_value** | **float** |  | [optional] 
**qdrant_point_id** | **str** |  | 
**tag_set** | **List[str]** |  | [optional] 
**time_stamp** | **datetime** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**weight** | **float** |  | 

## Example

```python
from trieve_py_client.models.qdrant_chunk_metadata import QdrantChunkMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of QdrantChunkMetadata from a JSON string
qdrant_chunk_metadata_instance = QdrantChunkMetadata.from_json(json)
# print the JSON string representation of the object
print(QdrantChunkMetadata.to_json())

# convert the object into a dict
qdrant_chunk_metadata_dict = qdrant_chunk_metadata_instance.to_dict()
# create an instance of QdrantChunkMetadata from a dict
qdrant_chunk_metadata_form_dict = qdrant_chunk_metadata.from_dict(qdrant_chunk_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


