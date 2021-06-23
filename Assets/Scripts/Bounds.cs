using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Bounds : MonoBehaviour
{
    public MinMaxF boundsX; 
    public MinMaxF boundsY; 

    public BoundaryParams config;

    void Awake()
    {
        config = GameObject.Find("Settings").GetComponent<Settings>().boundaryParams;

        var left   = transform.Find("left");
        var right  = transform.Find("right");
        var bottom = transform.Find("bottom");
        var top    = transform.Find("top");

        left.transform.Translate(-config.width/2, 0, 0);
        right.transform.Translate(config.width/2, 0, 0);
        bottom.transform.Translate(0, -config.height/2, 0);
        top.transform.Translate(0, config.height/2, 0);
        
        left.transform.localScale = new Vector3(config.thickness, config.height, 0);
        right.transform.localScale = new Vector3(config.thickness, config.height, 0);
        bottom.transform.localScale = new Vector3(config.width, config.thickness, 0);
        top.transform.localScale = new Vector3(config.width, config.thickness, 0);

        boundsX.min = -config.width/2 + config.thickness/2;
        boundsX.max =  config.width/2 - config.thickness/2;
        boundsY.min = -config.height/2 + config.thickness/2;
        boundsY.max =  config.height/2 - config.thickness/2;
    }

    public Vector3 GetRandomPos()
    {
        return new Vector3(boundsX.sample(), boundsY.sample(), 0.0f);
    }
}
