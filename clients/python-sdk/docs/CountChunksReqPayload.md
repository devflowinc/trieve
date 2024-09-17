# CountChunksReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**limit** | **int** | Set limit to restrict the maximum number of chunks to count. This is useful for when you want to reduce the latency of the count operation. By default the limit will be the number of chunks in the dataset. | [optional] 
**query** | [**QueryTypes**](QueryTypes.md) |  | 
**score_threshold** | **float** | Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0. | [optional] 
**search_type** | [**CountSearchMethod**](CountSearchMethod.md) |  | 
**use_quote_negated_terms** | **bool** | If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false. | [optional] 

## Example

```python
from trieve_py_client.models.count_chunks_req_payload import CountChunksReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CountChunksReqPayload from a JSON string
count_chunks_req_payload_instance = CountChunksReqPayload.from_json(json)
# print the JSON string representation of the object
print(CountChunksReqPayload.to_json())

# convert the object into a dict
count_chunks_req_payload_dict = count_chunks_req_payload_instance.to_dict()
# create an instance of CountChunksReqPayload from a dict
count_chunks_req_payload_form_dict = count_chunks_req_payload.from_dict(count_chunks_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


