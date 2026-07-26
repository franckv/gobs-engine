import common;

// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
// Ported from https://www.shadertoy.com/view/Md2SR3


[[vk::binding(0, COMPUTE_DATA_SET)]]
[format("rgba16f")]
RWTexture2D<float4> image;

// Return random noise in the range [0.0, 1.0], as a function of x.
float Noise2d( float2 x ) {
    float xhash = cos( x.x * 37.0 );
    float yhash = cos( x.y * 57.0 );
    return frac( 415.92653 * ( xhash + yhash ) );
}

// Convert Noise2d() into a "star field" by stomping everthing below fThreshhold to zero.
float NoisyStarField( float2 vSamplePos, float fThreshhold ) {
    float StarVal = Noise2d( vSamplePos );
    if ( StarVal >= fThreshhold )
        StarVal = pow( (StarVal - fThreshhold)/(1.0 - fThreshhold), 6.0 );
    else
        StarVal = 0.0;
    return StarVal;
}

// Stabilize NoisyStarField() by only sampling at integer values.
float StableStarField( float2 vSamplePos, float fThreshhold ) {
    // Linear interpolation between four samples.
    // Note: This approach has some visual artifacts.
    // There must be a better way to "anti alias" the star field.
    float fractX = frac( vSamplePos.x );
    float fractY = frac( vSamplePos.y );
    float2 floorSample = floor( vSamplePos );
    float v1 = NoisyStarField( floorSample, fThreshhold );
    float v2 = NoisyStarField( floorSample + float2( 0.0, 1.0 ), fThreshhold );
    float v3 = NoisyStarField( floorSample + float2( 1.0, 0.0 ), fThreshhold );
    float v4 = NoisyStarField( floorSample + float2( 1.0, 1.0 ), fThreshhold );

    float StarVal =   v1 * ( 1.0 - fractX ) * ( 1.0 - fractY )
        + v2 * ( 1.0 - fractX ) * fractY
        + v3 * fractX * ( 1.0 - fractY )
        + v4 * fractX * fractY;
	return StarVal;
}

void mainImage( out float4 fragColor,  float2 fragCoord, float2 iResolution ) {
	// Sky Background Color
	float3 vColor = float3( 0.1, 0.2, 0.4 ) * fragCoord.y / iResolution.y;

    // Note: Choose fThreshhold in the range [0.99, 0.9999].
    // Higher values (i.e., closer to one) yield a sparser starfield.
    float StarFieldThreshhold = 0.97;

    // Stars with a slow crawl.
    float xRate = 0.2;
    float yRate = -0.06;
    float2 vSamplePos = fragCoord.xy + float2( xRate , yRate );
	float StarVal = StableStarField( vSamplePos, StarFieldThreshhold );
    vColor += float3( StarVal, StarVal, StarVal );
	fragColor = float4(vColor, 1.0);
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 dispatchThreadId: SV_DispatchThreadID) {
	float4 value = float4(0.0, 0.0, 0.0, 1.0);
    int2 texelCoord = int2(dispatchThreadId.xy);

    uint width, height;
    image.GetDimensions(width, height);
	int2 size = int2(width, height);

    if(texelCoord.x < size.x && texelCoord.y < size.y)
    {
        float4 color;
        mainImage(color, float2(texelCoord), float2(size));

        image[texelCoord] = color;
    }
}
